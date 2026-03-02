import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/trackview.dart';
import 'package:ui/src/screens/desktop/central_panel.dart';

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
          child: GraphicsPadding(
            child: TrackView(
              rendererParameters: RendererParameters(
                kinds: {Kind.userStep},
                trackData: TrackData.map,
              ),
            ),
          ),
        ),
        Expanded(
          child: GraphicsPadding(
            child: TrackView(
              rendererParameters: RendererParameters(
                kinds: {Kind.userStep},
                trackData: TrackData.wheel,
              ),
            ),
          ),
        ),
      ],
    );

    List<Widget> children = [
      ConstrainedBox(
        constraints: BoxConstraints(minHeight: 275, maxHeight: 275),
        child: ProfilePadding(
          child: TrackView(
            rendererParameters: RendererParameters(
              kinds: {Kind.userStep},
              trackData: TrackData.profile,
            ),
          ),
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
  const CentralPanelUserSteps({super.key, required this.width});

  @override
  Widget build(BuildContext context) {
    return CentralPanelContent(
      width: width,
      trackData: [TrackData.map, TrackData.profile, TrackData.wheel],
      screenFocus: ScreenFocus.usersteps,
      child: CentralWidget(width: width),
    );
  }
}
