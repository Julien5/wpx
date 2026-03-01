import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/trackview.dart';
import 'package:ui/src/screens/desktop/central_panel.dart';

class CentralPanelPDF extends StatefulWidget {
  final double width;
  const CentralPanelPDF({super.key, required this.width});

  @override
  State<CentralPanelPDF> createState() => _CentralPanelPDFState();
}

class _CentralPanelPDFState extends State<CentralPanelPDF> {
  FutureRenderer? futureRenderer;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (futureRenderer == null) {
      SegmentModel segmentModel = Provider.of(context);
      futureRenderer = FutureRenderer(
        bridge: segmentModel.backend,
        segment: segmentModel.segment,
        trackData: [TrackData.map, TrackData.profile, TrackData.wheelPages],
        kinds: allkinds(),
      );
    }

    // this panel should be shown only if the focus is on pdf
    FociModel fociModel = Provider.of(context, listen: false);
    bool isVisible = fociModel.contains(ScreenFocus.settings);
    if (!isVisible) {
      return;
    }

    futureRenderer!.restart();
  }

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
                kinds: allkinds(),
                trackData: TrackData.map,
              ),
            ),
          ),
        ),
        Expanded(
          child: GraphicsPadding(
            child: TrackView(
              rendererParameters: RendererParameters(
                kinds: allkinds(),
                trackData: TrackData.wheelPages,
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
      Expanded(child: bottom),
    ];

    Widget child = ConstrainedBox(
      constraints: BoxConstraints(maxWidth: widget.width),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        mainAxisAlignment: MainAxisAlignment.end,
        children: children,
      ),
    );

    return _Provider(futureRenderer: futureRenderer!, child: child);
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
