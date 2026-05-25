import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/futurerenderer.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/widgets/trackview.dart';
import 'package:wpx/src/screens/desktop/central_panel.dart';
import 'package:wpx/src/widgets/waypoints_table_widget.dart';

class CentralWidget extends StatelessWidget {
  final double width;
  const CentralWidget({super.key, required this.width});

  @override
  Widget build(BuildContext context) {
    Provider.of<ParameterModel>(context);
    Widget bottom = Row(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Expanded(
          child: GraphicsPadding(child: TrackView(trackData: TrackData.map)),
        ),
        Expanded(child: GPXTable(kinds: [Kind.cutOff])),
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

class CentralPanelUserSteps extends StatelessWidget {
  final double width;
  const CentralPanelUserSteps({super.key, required this.width});

  @override
  Widget build(BuildContext context) {
    Kinds kinds = allkinds();
    kinds.add(Kind.cutOff);
    return CentralPanelContent(
      width: width,
      clients: [TrackData.map, TrackData.profile],
      screenFocus: ScreenFocus.usersteps,
      child: CentralWidget(width: width),
    );
  }
}
