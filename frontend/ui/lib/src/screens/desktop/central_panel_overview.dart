import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/trackview.dart';
import 'package:ui/src/screens/desktop/central_panel.dart';

class CentralPanelOverview extends StatefulWidget {
  final double width;
  const CentralPanelOverview({super.key, required this.width});

  @override
  State<CentralPanelOverview> createState() => _CentralPanelOverviewState();
}

class _CentralPanelOverviewState extends State<CentralPanelOverview> {
  FutureRenderer? futureRenderer;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (futureRenderer == null) {
      debugPrint("BUILD FUTURE RENDERER OVERVIEW PANEL");
      SegmentModel segmentModel = Provider.of(context);
      futureRenderer = FutureRenderer(
        bridge: segmentModel.backend,
        segment: segmentModel.segment,
        trackData: [TrackData.map, TrackData.profile],
        kinds: allkinds(),
      );
    }

    // this panel should be shown only if the focus is on pdf
    FociModel fociModel = Provider.of(context, listen: false);
    bool isVisible =
        !fociModel.contains(ScreenFocus.settings) &&
        !fociModel.contains(ScreenFocus.usersteps);
    if (!isVisible) {
      return;
    }
    futureRenderer!.restart();
  }

  @override
  Widget build(BuildContext context) {
    Provider.of<ParameterModel>(context);
    Widget maprow = Row(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Expanded(
          child: GraphicsPadding(
            child: TrackView(
              rendererParameters: RendererParameters(
                kinds: allkinds(),
                trackData: TrackData.map,
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
              kinds: allkinds(),
              trackData: TrackData.profile,
            ),
          ),
        ),
      ),
      Expanded(child: maprow),
    ];

    Widget child = ConstrainedBox(
      constraints: BoxConstraints(maxWidth: widget.width),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        mainAxisAlignment: MainAxisAlignment.end,
        children: children,
      ),
    );

    return _CentralPanelOverviewProvider(
      futureRenderer: futureRenderer!,
      child: child,
    );
  }
}

class _CentralPanelOverviewProvider extends StatelessWidget {
  final FutureRenderer futureRenderer;
  final Widget child;

  const _CentralPanelOverviewProvider({
    required this.futureRenderer,
    required this.child,
  });

  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [ChangeNotifierProvider.value(value: futureRenderer)],
      child: child,
    );
  }
}
