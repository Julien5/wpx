import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/futurerenderer.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/widgets/trackview.dart';
import 'package:wpx/src/screens/desktop/central_panel.dart';
import 'package:wpx/src/widgets/waypoints_table_widget.dart';

class CentralWidget extends StatefulWidget {
  final double width;
  const CentralWidget({super.key, required this.width});

  @override
  State<CentralWidget> createState() => _CentralWidgetState();
}

class _CentralWidgetState extends State<CentralWidget> {
  Widget? table;

  @override
  Widget build(BuildContext context) {
    Provider.of<ParameterModel>(context);
    FutureRenderer renderer = Provider.of<FutureRenderer>(context);
    RenderOutput? renderOutput = renderer.renderOutput(RenderFunction.profile);
    table ??= Text("no waypoints");
    if (renderOutput != null) {
      List<Waypoint> waypoints = decimate(
        waypoints: renderOutput.waypoints,
        segment: renderer.getSegment(),
        n: BigInt.from(renderOutput.waypoints.length),
      );
      table = DesktopTable(waypoints: waypoints, editControls: true);
    }
    Widget bottom = Row(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Expanded(
          child: GraphicsPadding(child: TrackView(trackData: TrackData.map)),
        ),
        Expanded(child: table!),
      ],
    );

    List<Widget> children = [
      ConstrainedBox(
        constraints: BoxConstraints(minHeight: 275, maxHeight: 275),
        child: ProfilePadding(child: TrackView(trackData: TrackData.profile)),
      ),
      Expanded(child: bottom),
    ];

    return ConstrainedBox(
      constraints: BoxConstraints(maxWidth: widget.width),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        mainAxisAlignment: MainAxisAlignment.end,
        children: children,
      ),
    );
  }
}

class CentralPanelOverview extends StatelessWidget {
  final double width;
  const CentralPanelOverview({super.key, required this.width});

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (BuildContext context, BoxConstraints constraints) {
        return CentralPanelContent(
          width: width,
          clients: [TrackData.profile, TrackData.map],
          screenFocus: ScreenFocus.overview,
          child: CentralWidget(width: width),
        );
      },
    );
  }
}
