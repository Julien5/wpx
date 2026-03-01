import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/trackview.dart';
import 'package:ui/src/screens/desktop/central_panel.dart';

class CentralPanelUserSteps extends StatefulWidget {
  final double width;
  const CentralPanelUserSteps({super.key, required this.width});

  @override
  State<CentralPanelUserSteps> createState() => _CentralPanelUserStepsState();
}

class _CentralPanelUserStepsState extends State<CentralPanelUserSteps> {
  FutureRenderer? futureRenderer;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();

    if (futureRenderer == null) {
      SegmentModel segmentModel = Provider.of(context);
      futureRenderer = FutureRenderer(
        bridge: segmentModel.backend,
        segment: segmentModel.segment,
        trackData: [TrackData.wheel],
        kinds: allkinds(),
      );
    }

    // this panel should be shown only if the focus is on user steps
    FociModel fociModel = Provider.of(context, listen: false);
    bool isVisible = fociModel.contains(ScreenFocus.usersteps);
    if (!isVisible) {
      return;
    }

    futureRenderer!.restart();
  }

  @override
  Widget build(BuildContext context) {
    Provider.of<ParameterModel>(context);
    Widget track = TrackView(
      rendererParameters: RendererParameters(
        kinds: allkinds(),
        trackData: TrackData.wheel,
      ),
    );
    return _Provider(
      futureRenderer: futureRenderer!,
      child: GraphicsPadding(child: track),
    );
  }
}

class _Provider extends StatelessWidget {
  final FutureRenderer futureRenderer;
  final Widget child;

  const _Provider({required this.futureRenderer, required this.child});

  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [ChangeNotifierProvider.value(value: futureRenderer)],
      child: child,
    );
  }
}
