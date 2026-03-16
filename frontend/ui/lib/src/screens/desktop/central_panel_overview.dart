import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/trackview.dart';
import 'package:ui/src/screens/desktop/central_panel.dart';
import 'package:ui/src/widgets/waypoints_table_widget.dart';

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
        Expanded(child: table),
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
