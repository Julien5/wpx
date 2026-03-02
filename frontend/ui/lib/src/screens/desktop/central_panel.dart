import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/screens/desktop/central_panel_overview.dart';
import 'package:ui/src/screens/desktop/central_panel_pdf.dart';
import 'package:ui/src/screens/desktop/central_panel_usersteps.dart';

class ProfilePadding extends StatelessWidget {
  final Widget child;

  const ProfilePadding({super.key, required this.child});
  @override
  Widget build(BuildContext ctx) {
    return Padding(
      padding: EdgeInsetsGeometry.fromLTRB(5, 10, 10, 10),
      child: child,
    );
  }
}

class GraphicsPadding extends StatelessWidget {
  final Widget child;

  const GraphicsPadding({super.key, required this.child});
  @override
  Widget build(BuildContext ctx) {
    return Padding(padding: EdgeInsetsGeometry.all(20), child: child);
  }
}

class CentralPanel extends StatefulWidget {
  final double width;
  const CentralPanel({super.key, required this.width});

  @override
  State<CentralPanel> createState() => _CentralPanelState();
}

class CentralPanelContent extends StatefulWidget {
  final double width;
  final List<TrackData> trackData;
  final ScreenFocus screenFocus;
  final Widget child;
  const CentralPanelContent({
    super.key,
    required this.width,
    required this.trackData,
    required this.screenFocus,
    required this.child,
  });

  @override
  State<CentralPanelContent> createState() => _CentralPanelContentState();
}

class _CentralPanelContentState extends State<CentralPanelContent> {
  FutureRenderer? futureRenderer;
  bool _needsRestart = false;

  bool isVisible(FociModel fociModel) {
    if (widget.screenFocus == ScreenFocus.overview) {
      return fociModel.hasOnly(ScreenFocus.overview);
    }
    return fociModel.contains(widget.screenFocus);
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    context.watch<ParameterModel>();
    if (futureRenderer == null) {
      SegmentModel segmentModel = Provider.of(context);
      futureRenderer = FutureRenderer(
        bridge: segmentModel.backend,
        segment: segmentModel.segment,
        trackData: widget.trackData,
        kinds: allkinds(),
      );
    }
    debugPrint("central panel update");
    _needsRestart = true;
  }

  @override
  Widget build(BuildContext context) {
    Provider.of<ParameterModel>(context);
    FociModel fociModel = Provider.of<FociModel>(context);
    assert(futureRenderer != null);
    if (isVisible(fociModel) && _needsRestart || futureRenderer!.needsStart()) {
      debugPrint("restart renderer in widget ${widget.screenFocus}");
      futureRenderer!.restart();
      _needsRestart = false;
    }

    return _Provider(futureRenderer: futureRenderer!, child: widget.child);
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

class _CentralPanelState extends State<CentralPanel> {
  @override
  Widget build(BuildContext context) {
    FociModel fociModel = Provider.of<FociModel>(context);

    return IndexedStack(
      index:
          fociModel.contains(ScreenFocus.usersteps)
              ? 0
              : fociModel.contains(ScreenFocus.settings)
              ? 1
              : 2,
      children: [
        SizedBox(
          width: widget.width,
          child: CentralPanelUserSteps(width: widget.width),
        ),
        SizedBox(
          width: widget.width,
          child: CentralPanelPDF(width: widget.width),
        ),
        SizedBox(
          width: widget.width,
          child: CentralPanelOverview(width: widget.width),
        ),
      ],
    );
  }
}
