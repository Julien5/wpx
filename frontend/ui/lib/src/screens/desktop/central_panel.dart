import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/futurerenderer.dart';
import 'package:wpx/src/models/kindsmodel.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/screens/desktop/central_panel_overview.dart';
import 'package:wpx/src/screens/desktop/central_panel_pdf.dart';
import 'package:wpx/src/screens/desktop/central_panel_usersteps.dart';
import 'package:wpx/src/utils/utils.dart';

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
  final ScreenFocus screenFocus;
  final Widget child;
  const CentralPanelContent({
    super.key,
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
    KindsModel kindsModel = Provider.of(context);
    if (futureRenderer == null) {
      SegmentModel segmentModel = Provider.of(context);
      debugPrint("CREATE FUTURE RENDER FOR ${widget.screenFocus}");
      futureRenderer = FutureRenderer(
        bridge: segmentModel.backend,
        segment: segmentModel.segment,
        clients: [RenderFunction.profile, RenderFunction.map],
        kinds: kindsModel.kinds,
        name: "${widget.screenFocus}",
      );
    } else {
      debugPrint("REUSE FUTURE RENDER FOR ${widget.screenFocus}");
    }
    futureRenderer!.setKinds(kindsModel.kinds);
    futureRenderer!.setVisible(isVisible(fociModel));
    // Needed because futureRenderer does not know the time parameters.
    // Change time parameters => update graphics.
    futureRenderer!.reset();
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
    return FutureRendererProvider(
      futureRenderer: futureRenderer!,
      child: widget.child,
    );
  }
}

class CentralPanelTabView extends StatefulWidget {
  final double width;
  final List<RenderFunction> clients;
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
  List<SegmentModel> segments = [];

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
    List<Segment> segs = root.backend.segments();
    if (renderers.length != segs.length) {
      disposeModels();
      RootModel root = Provider.of(context);
      for (int k = 0; k < segs.length; ++k) {
        renderers.add(
          FutureRenderer(
            bridge: root.backend,
            segment: segs[k],
            clients: widget.clients,
            kinds: widget.kinds,
            name: "${widget.screenFocus}",
          ),
        );
        segments.add(SegmentModel(segment: segs[k], backend: root.backend));
      }
    }
    KindsModel kindsModel = Provider.of(context);
    for (FutureRenderer renderer in renderers) {
      renderer.setKinds(kindsModel.kinds);
      renderer.setVisible(isVisible(fociModel));
      renderer.reset();
    }
  }

  void disposeModels() {
    for (FutureRenderer renderer in renderers) {
      renderer.dispose();
    }
    renderers.clear();
    for (SegmentModel segment in segments) {
      segment.dispose();
    }
    segments.clear();
  }

  @override
  void dispose() {
    disposeModels();
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
      Segment segment = segments[k].segment;
      SegmentStatistics stat = root.backend.segmentStatistics(segment: segment);
      debugPrint("MODEL segment: ${segment.id()}: ${statisticsString(stat)}");
      MultiProvider provider = MultiProvider(
        providers: [
          ChangeNotifierProvider.value(value: renderer),
          ChangeNotifierProvider.value(value: segments[k]),
        ],
        child: widget.child,
      );

      children.add(provider);
    }
    return TabBarView(controller: widget.tabController, children: children);
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
