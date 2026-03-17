import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/trackview.dart';
import 'package:ui/src/screens/desktop/central_panel.dart';
import 'package:ui/src/widgets/waypoints_table_widget.dart';

class KindsRow extends StatefulWidget {
  const KindsRow({super.key});

  @override
  State<KindsRow> createState() => _KindsRowState();
}

class _KindsRowState extends State<KindsRow> {
  void onControlsCheck(bool? checked) {
    FutureRenderer renderer = Provider.of(context, listen: false);
    if (checked == null || checked == false) {
      renderer.removeKind(Kind.controls);
    } else {
      renderer.addKind(Kind.controls);
    }
  }

  void onWaypointsCheck(bool? checked) {
    FutureRenderer renderer = Provider.of(context, listen: false);
    if (checked == null || checked == false) {
      renderer.removeKind(Kind.gpxWaypoints);
    } else {
      renderer.addKind(Kind.gpxWaypoints);
    }
  }

  void onOSMCheck(bool? checked) {
    FutureRenderer renderer = Provider.of(context, listen: false);
    if (checked == null || checked == false) {
      renderer.removeKind(Kind.cities);
      renderer.removeKind(Kind.villages);
      renderer.removeKind(Kind.hamlets);
      renderer.removeKind(Kind.mountains);
    } else {
      renderer.addKind(Kind.cities);
      renderer.addKind(Kind.villages);
      renderer.addKind(Kind.hamlets);
      renderer.addKind(Kind.mountains);
    }
  }

  @override
  Widget build(BuildContext context) {
    FutureRenderer renderer = Provider.of(context);
    bool hasControls = renderer.kinds.contains(Kind.controls);
    bool hasGPXWaypoints = renderer.kinds.contains(Kind.gpxWaypoints);
    bool hasCities = renderer.kinds.contains(Kind.cities);
    SizedBox hdiv = SizedBox(width: 10);
    return Row(
      children: [
        Checkbox(
          tristate: true,
          value: hasControls,
          onChanged: onControlsCheck,
        ),
        Text("Controls"),
        hdiv,
        Checkbox(
          tristate: true,
          value: hasGPXWaypoints,
          onChanged: onWaypointsCheck,
        ),
        Text("Waypoints"),
        hdiv,
        Checkbox(tristate: true, value: hasCities, onChanged: onOSMCheck),
        Text("OSM"),
      ],
    );
  }
}

class CentralWidget extends StatelessWidget {
  final double width;
  const CentralWidget({super.key, required this.width});

  @override
  Widget build(BuildContext context) {
    Provider.of<ParameterModel>(context);
    FutureRenderer renderer = Provider.of<FutureRenderer>(context);
    RenderOutput? renderOutput = renderer.renderOutput(RenderFunction.profile);
    Widget table = Text("no waypoints");
    if (renderOutput != null) {
      table = DesktopTable(waypoints: renderOutput.waypoints);
    }
    Widget bottom = Row(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Expanded(
          child: GraphicsPadding(child: TrackView(trackData: TrackData.map)),
        ),
        Expanded(child: Column(children: [KindsRow(), table])),
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
      constraints: BoxConstraints(maxWidth: width),
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
          kinds: allkinds(),
          screenFocus: ScreenFocus.overview,
          child: CentralWidget(width: width),
        );
      },
    );
  }
}
