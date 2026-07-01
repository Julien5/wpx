import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/futurerenderer.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/utils/utils.dart';
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
    FutureRenderer renderer = context.watch<FutureRenderer>();
    RenderOutput? renderOutput = renderer.renderOutput(RenderFunction.profile);
    table ??= Text("no waypoints");
    SegmentModel segmentModel = context.watch();
    List<Waypoint> waypoints = getBackend(context).getWaypoints(
      segment: segmentModel.segment,
      kinds: [Kind.gpxWaypoints, Kind.controls],
    );
    if (renderOutput != null) {
      if (waypoints.length < 15) {
        List<Waypoint> osm =
            renderOutput.waypoints
                .where(
                  (waypoint) =>
                      waypoint.origin != Kind.controls &&
                      waypoint.origin != Kind.gpxWaypoints,
                )
                .toList();
        osm = decimate(
          waypoints: osm,
          segment: renderer.getSegment(),
          n: BigInt.from(15),
        );
        waypoints.addAll(osm);
        waypoints.sort((w1, w2) {
          return w1.info!.distance.compareTo(w2.info!.distance);
        });
      }
      table = DesktopTable(waypoints: waypoints, editControls: true);
    } else {
      // build() is also triggered from futureRenderer!.reset() in
      // see CentralPanelContent, didChangeDependencies.
    }

    Widget bottom = Row(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Expanded(
          child: GraphicsPadding(
            child: TrackView(trackData: RenderFunction.map),
          ),
        ),
        Expanded(child: Center(child: table!)),
      ],
    );

    List<Widget> children = [
      ConstrainedBox(
        constraints: BoxConstraints(minHeight: 275, maxHeight: 275),
        child: ProfilePadding(
          child: TrackView(trackData: RenderFunction.profile),
        ),
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
          screenFocus: ScreenFocus.overview,
          child: CentralWidget(width: width),
        );
      },
    );
  }
}
