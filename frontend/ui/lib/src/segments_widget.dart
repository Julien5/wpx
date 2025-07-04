import 'dart:developer' as developer;

import 'package:file_picker/file_picker.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/counter.dart';
import 'package:ui/src/segment_stack.dart';

class RenderingsProvider extends MultiProvider {
  final Renderers renderers;

  RenderingsProvider(Renderers r, Widget child, {super.key})
    : renderers = r,
      super(
        providers: [
          ChangeNotifierProvider.value(value: r.trackRendering),
          ChangeNotifierProvider.value(value: r.waypointsRendering),
        ],
        child: child,
      );
}

class SegmentsView extends StatelessWidget {
  final SegmentsProvider? segmentsProvider;
  const SegmentsView({super.key, this.segmentsProvider});

  @override
  Widget build(BuildContext context) {
    var S = segmentsProvider!.segments();
    List<RenderingsProvider> segments = [];
    assert(segments.isEmpty);
    for (var renderer in S) {
      var w = RenderingsProvider(renderer, SegmentStack());
      w.renderers.trackRendering.start();
      segments.add(w);
    }

    developer.log("[segments] [build] #segments=${segments.length}");
    List<Tab> tabs = [];
    for (var s in segments) {
      var id = s.renderers.trackRendering.segment.id();
      tabs.add(Tab(text: "segment ${id.toInt()}"));
    }
    return MaterialApp(
      home: DefaultTabController(
        length: segments.length,
        child: Scaffold(
          appBar: AppBar(bottom: TabBar(tabs: tabs)),
          body: TabBarView(children: segments),
        ),
      ),
    );
  }
}

class FindGPXFile extends StatelessWidget {
  final SegmentsProvider segmentsProvider;
  const FindGPXFile({super.key, required this.segmentsProvider});

  void onPressed() async {
    FilePickerResult? result = await FilePicker.platform.pickFiles(
      type: FileType.any,
    );
    if (result != null) {
      developer.log("result: ${result.count}");
      for (var file in result.files) {
        if (!kIsWeb) {
          segmentsProvider.setFilename(file.path!);
        } else {
          segmentsProvider.setContent(file.bytes!.buffer.asInt8List().toList());
        }
        return;
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          ElevatedButton(
            onPressed: onPressed,
            child: const Text("Choose GPX file"),
          ),
        ],
      ),
    );
  }
}

class SegmentsConsumer extends StatelessWidget {
  const SegmentsConsumer({super.key});

  @override
  Widget build(BuildContext ctx) {
    return Consumer<SegmentsProvider>(
      builder: (context, segmentsProvider, child) {
        developer.log("length=${segmentsProvider.segments().length}");
        return Center(
          child: Container(
            constraints: const BoxConstraints(maxWidth: 1500),
            child: Column(
              children: [
                FindGPXFile(segmentsProvider: segmentsProvider),
                Buttons(
                  more: segmentsProvider.decrementDelta,
                  less: segmentsProvider.incrementDelta,
                ),
                Expanded(
                  child: SegmentsView(segmentsProvider: segmentsProvider),
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}

class Buttons extends StatelessWidget {
  final VoidCallback more;
  final VoidCallback less;
  const Buttons({super.key, required this.more, required this.less});

  @override
  Widget build(BuildContext ctx) {
    return Column(
      children: [
        Row(
          children: [
            PressButton(label: "more", onCounterPressed: more),
            PressButton(label: "less", onCounterPressed: less),
          ],
        ),
      ],
    );
  }
}
