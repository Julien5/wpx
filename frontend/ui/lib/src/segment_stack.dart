import 'dart:developer' as developer;
import 'dart:io';

import 'package:file_picker/file_picker.dart';
import 'package:file_saver/file_saver.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/future_rendering_widget.dart';
import 'package:ui/src/settings_widget.dart';
import 'package:ui/src/waypoints_widget.dart';
import 'package:visibility_detector/visibility_detector.dart';

class TestPdfButton extends StatefulWidget {
  const TestPdfButton({super.key});

  @override
  State<TestPdfButton> createState() => _TestPdfButtonState();
}

void savePdf(List<int> data) async {
  if (kIsWeb) {
    await FileSaver.instance.saveFile(
      name: "waypoints",
      bytes: Uint8List.fromList(data),
      fileExtension: "pdf",
      mimeType: MimeType.pdf,
    );
  } else if (Platform.isLinux) {
    var filepath = await FilePicker.platform.saveFile(
      fileName: "waypoints",
      type: FileType.custom,
      allowedExtensions: ["pdf"],
      bytes: Uint8List.fromList(data),
    );
    if (filepath == null) {
      return;
    }
    await Process.run('xdg-open', [filepath]);
  }
}

class _TestPdfButtonState extends State<TestPdfButton> {
  int pdflength = 0;

  void onPressed(SegmentsProvider model) async {
    if (!mounted) {
      return;
    }
    setState(() {
      pdflength = 0;
    });
    var data = await model.generatePdf();
    savePdf(data);
    setState(() {
      developer.log("pdf length: ${data.length}");
      pdflength = data.length;
    });
  }

  @override
  Widget build(BuildContext context) {
    SegmentsProvider model = Provider.of<SegmentsProvider>(context);
    return Row(
      children: [
        ElevatedButton(onPressed: () => onPressed(model), child: Text("PDF")),
        Text("length: $pdflength"),
      ],
    );
  }
}

class SegmentStack extends StatelessWidget {
  const SegmentStack({super.key});

  @override
  Widget build(BuildContext context) {
    double width = MediaQuery.sizeOf(context).width;
    double height = MediaQuery.sizeOf(context).height;
    developer.log("[stack] ${width}x$height");
    var stack = Align(
      alignment: Alignment.center, // Center the Stack horizontally
      child: Stack(children: <Widget>[TrackConsumer(), WaypointsConsumer()]),
    );
    var wp = SizedBox(height: 150, child: WayPointsConsumer());
    return Column(
      children: [
        stack,
        Row(children: [WidthSettings(), TestPdfButton()]),
        wp,
      ],
    );
  }
}

class TrackConsumer extends StatelessWidget {
  const TrackConsumer({super.key});

  @override
  Widget build(BuildContext ctx) {
    return Consumer<TrackRenderer>(
      builder: (context, trackRenderer, child) {
        return FutureRenderingWidget(future: trackRenderer);
      },
    );
  }
}

class WaypointsConsumer extends StatefulWidget {
  const WaypointsConsumer({super.key});

  @override
  State<WaypointsConsumer> createState() => _WaypointsConsumerState();
}

class _WaypointsConsumerState extends State<WaypointsConsumer> {
  double visibility = 0;

  void onVisibilityChanged(VisibilityInfo info) {
    if (!mounted) {
      return;
    }
    WaypointsRenderer wp = Provider.of<WaypointsRenderer>(
      context,
      listen: false,
    );
    developer.log(
      "[waypoint consumer] id:${wp.id()} vis:${info.visibleFraction}",
    );
    wp.updateVisibility(info.visibleFraction);
  }

  @override
  Widget build(BuildContext ctx) {
    return Consumer<WaypointsRenderer>(
      builder: (context, waypointsRenderer, child) {
        // It would be more accurate to check visibility with a scroll controller
        // at the list view level. Because "Callbacks are not fired immediately
        // on visibility changes."
        return VisibilityDetector(
          key: Key('id:${waypointsRenderer.id()}'),
          onVisibilityChanged: onVisibilityChanged,
          child: FutureRenderingWidget(future: waypointsRenderer),
        );
      },
    );
  }
}
