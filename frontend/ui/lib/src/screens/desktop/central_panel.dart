import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/screens/desktop/central_panel_overview.dart';
import 'package:ui/src/screens/desktop/central_panel_pdf.dart';
import 'package:ui/src/screens/desktop/central_panel_usersteps.dart';
import 'package:ui/utils.dart';

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

class CentralPanelContent extends StatefulWidget {
  final double width;
  final List<TrackData> clients;
  final Kinds kinds;
  final ScreenFocus screenFocus;
  final Widget child;
  const CentralPanelContent({
    super.key,
    required this.width,
    required this.clients,
    required this.kinds,
    required this.screenFocus,
    required this.child,
  });

  @override
  State<CentralPanelContent> createState() => _CentralPanelContentState();
}

class _CentralPanelContentState extends State<CentralPanelContent> {
  FutureRenderer? futureRenderer;

  bool isVisible(FociModel fociModel) {
    if (widget.screenFocus == ScreenFocus.overview) {
      return fociModel.hasOnly(ScreenFocus.overview);
    }
    return fociModel.contains(widget.screenFocus);
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    FociModel fociModel = context.watch<FociModel>();
    context.watch<ParameterModel>();
    if (futureRenderer == null) {
      SegmentModel segmentModel = Provider.of(context);
      futureRenderer = FutureRenderer(
        bridge: segmentModel.backend,
        segment: segmentModel.segment,
        clients: widget.clients,
        kinds: widget.kinds,
      );
    }
    debugPrint("central panel: reset renderer");
    futureRenderer!.setVisible(isVisible(fociModel));
  }

  @override
  void dispose() {
    futureRenderer?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    debugPrint("_CentralPanelContentState(${futureRenderer!.clients}) build()");
    assert(futureRenderer != null);
    // Rendering is done on "each frame". This works because the build() function
    // is not called often, only when one of the dependency has changed
    // (ParameterModel, FociModel) or the window has a new size. Which are exactly
    // the situations where we need to recompute the graphics.
    // However, we still need to maintain the renderer in the state, because it
    // works asynchronously and must be kept between frames.
    futureRenderer!.reset();
    return _Provider(futureRenderer: futureRenderer!, child: widget.child);
  }
}

class CentralPanelTabView extends StatefulWidget {
  final double width;
  final List<TrackData> clients;
  final Kinds kinds;
  final ScreenFocus screenFocus;
  final TabController tabController;
  final Widget child;
  const CentralPanelTabView({
    super.key,
    required this.width,
    required this.clients,
    required this.kinds,
    required this.screenFocus,
    required this.tabController,
    required this.child,
  });

  @override
  State<CentralPanelTabView> createState() => _CentralPanelTabViewState();
}

class _CentralPanelTabViewState extends State<CentralPanelTabView> {
  // the list cannot be empty, so empty marks uninitialized
  List<FutureRenderer> renderers = [];
  List<Segment> segments = [];

  bool isVisible(FociModel fociModel) {
    if (widget.screenFocus == ScreenFocus.overview) {
      return fociModel.hasOnly(ScreenFocus.overview);
    }
    return fociModel.contains(widget.screenFocus);
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    FociModel fociModel = context.watch<FociModel>();
    context.watch<ParameterModel>();
    RootModel root = Provider.of(context);
    segments = root.backend.segments();
    if (renderers.length != segments.length) {
      disposeRenderers();
      RootModel root = Provider.of(context);
      assert(segments.isNotEmpty);
      for (int k = 0; k < segments.length; ++k) {
        renderers.add(
          FutureRenderer(
            bridge: root.backend,
            segment: segments[k],
            clients: widget.clients,
            kinds: widget.kinds,
          ),
        );
      }
    }
    debugPrint("_CentralPanelTabViewState: ${segments.length} segments");
    debugPrint("central panel: reset renderer");
    for (FutureRenderer renderer in renderers) {
      renderer.setVisible(isVisible(fociModel));
    }
  }

  void disposeRenderers() {
    for (FutureRenderer renderer in renderers) {
      renderer.dispose();
    }
    renderers.clear();
  }

  @override
  void dispose() {
    disposeRenderers();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    debugPrint("CentralPanelTabView build()");
    assert(renderers.isNotEmpty);
    RootModel root = Provider.of<RootModel>(context, listen: false);
    Provider.of<ParameterModel>(context);
    List<Widget> children = [];
    for (int k = 0; k < segments.length; ++k) {
      FutureRenderer renderer = renderers[k];
      Segment segment = segments[k];
      renderer.reset();
      SegmentStatistics stat = root.backend.segmentStatistics(segment: segment);
      debugPrint(
        "CREATE MODEL segment: ${segment.id()}: ${statisticsString(stat)}",
      );
      MultiProvider provider = MultiProvider(
        providers: [
          ChangeNotifierProvider.value(value: renderer),
          ChangeNotifierProvider(
            create:
                (_) => SegmentModel(segment: segment, backend: root.backend),
          ),
        ],
        child: widget.child,
      );

      children.add(provider);
    }
    return TabBarView(controller: widget.tabController, children: children);
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

class CentralPanel extends StatefulWidget {
  final double width;
  const CentralPanel({super.key, required this.width});

  @override
  State<CentralPanel> createState() => _CentralPanelState();
}

class _CentralPanelState extends State<CentralPanel> {
  @override
  Widget build(BuildContext context) {
    FociModel fociModel = Provider.of<FociModel>(context);
    debugPrint("_CentralPanelState build()");
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
