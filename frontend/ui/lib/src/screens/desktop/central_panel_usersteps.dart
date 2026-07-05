import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
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
    context.watch<SegmentModel>();
    Widget bottom = Row(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Expanded(
          child: GraphicsPadding(
            child: TrackView(trackData: RenderFunction.map),
          ),
        ),
        Expanded(child: GPXTable(kinds: [Kind.cutOff])),
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
  final bool visible;
  const CentralPanelUserSteps({super.key, required this.width, required this.visible});

  @override
  Widget build(BuildContext context) {
    return CentralPanelContent(
      label: 'usersteps',
      visible: visible,
      child: CentralWidget(width: width),
    );
  }
}
