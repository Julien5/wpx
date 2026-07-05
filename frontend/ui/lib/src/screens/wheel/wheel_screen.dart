import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/routes.dart';
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/screens/wheel/statistics_widget.dart';
import 'package:wpx/src/widgets/adaptive_layout.dart';
import 'package:wpx/src/widgets/editable_text.dart';
import 'package:wpx/src/widgets/export.dart';
import 'package:wpx/src/widgets/segmentgraphics.dart';

class _WheelScaffold extends StatelessWidget {
  void onPDFPressed(BuildContext ctx) {
    gotoPDF(ctx);
  }

  void onPacingPointPressed(BuildContext ctx) {
    gotoUserSteps(ctx);
  }

  void onControlsPointPressed(BuildContext ctx) {
    gotoControls(ctx);
  }

  Future<void> updateTrackFileName(BuildContext ctx, String newName) async {
    ctx.read<SegmentModel>().updateTrackfileName(newName: newName);
  }

  @override
  Widget build(BuildContext ctx) {
    Widget statisticsCard = OverviewWidget(
      onPacingPointPressed: () => onPacingPointPressed(ctx),
      onControlsPointPressed: () => onControlsPointPressed(ctx),
      onPDFPressed: () => onPDFPressed(ctx),
    );
    List<Widget> children = [
      statisticsCard,
      Center(child: ExportButton(text: "export zip")),
    ];
    SegmentModel segmentModel = ctx.watch<SegmentModel>();
    if (!ctx.read<RootModel>().isLoaded()) {
      return Text("ignore");
    }
    String trackName = segmentModel.trackFileName();

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: Icon(Icons.home),
          onPressed: () => gotoHome(ctx),
        ),
        title: WritableText(
          initialName: trackName,
          onSubmitted:
              (newName) async => await updateTrackFileName(ctx, newName),
        ),
      ),
      body: MobileScaffoldBody(
        topRow: TrackGraphicsRow(kinds: allkinds()),
        midColumn: MidColumn(children: children),
        label: 'overview',
        clients: [
          RenderFunction.profile,
          RenderFunction.map,
          RenderFunction.wheel,
        ],
      ),
    );
  }
}

class WheelScreen extends StatelessWidget {
  const WheelScreen({super.key});

  @override
  Widget build(BuildContext context) {
    context.watch<SegmentModel>();
    return _WheelScaffold();
  }
}
