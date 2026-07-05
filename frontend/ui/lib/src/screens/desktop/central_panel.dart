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
  final String label;
  final bool visible;
  final Widget child;
  const CentralPanelContent({
    super.key,
    required this.label,
    required this.visible,
    required this.child,
  });

  @override
  State<CentralPanelContent> createState() => _CentralPanelContentState();
}

class _CentralPanelContentState extends State<CentralPanelContent> {
  FutureRenderer? futureRenderer;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    context.watch<SegmentModel>();
    KindsModel kindsModel = context.watch();
    if (futureRenderer == null) {
      SegmentModel segmentModel = context.watch();
      debugPrint("CREATE FUTURE RENDER FOR ${widget.label}");
      futureRenderer = FutureRenderer(
        bridge: segmentModel.backend,
        segment: segmentModel.segment,
        clients: [RenderFunction.profile, RenderFunction.map],
        kinds: kindsModel.kinds,
        name: widget.label,
      );
    } else {
      debugPrint("REUSE FUTURE RENDER FOR ${widget.label}");
    }
    futureRenderer!.setKinds(kindsModel.kinds);
    futureRenderer!.setVisible(widget.visible);
    futureRenderer!.reset();
  }

  @override
  void didUpdateWidget(CentralPanelContent old) {
    super.didUpdateWidget(old);
    if (old.visible != widget.visible && futureRenderer != null) {
      futureRenderer!.setVisible(widget.visible);
    }
  }

  @override
  void dispose() {
    futureRenderer?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    debugPrint("_CentralPanelContentState(${widget.label}) build()");
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
  final bool visible;
  final TabController tabController;
  final Widget child;
  const CentralPanelTabView({
    super.key,
    required this.width,
    required this.clients,
    required this.kinds,
    required this.visible,
    required this.tabController,
    required this.child,
  });

  @override
  State<CentralPanelTabView> createState() => _CentralPanelTabViewState();
}

class _CentralPanelTabViewState extends State<CentralPanelTabView> {
  List<FutureRenderer> renderers = [];
  List<SegmentModel> segments = [];

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    context.watch<SegmentModel>();
    RootModel root = context.read<RootModel>();
    List<Segment> segs = root.backend.segments();
    if (renderers.length != segs.length) {
      disposeModels();
      for (int k = 0; k < segs.length; ++k) {
        renderers.add(
          FutureRenderer(
            bridge: root.backend,
            segment: segs[k],
            clients: widget.clients,
            kinds: widget.kinds,
            name: "settings",
          ),
        );
        segments.add(SegmentModel(segment: segs[k], backend: root.backend));
      }
    }
    KindsModel kindsModel = context.watch();
    for (FutureRenderer renderer in renderers) {
      renderer.setKinds(kindsModel.kinds);
      renderer.setVisible(widget.visible);
      renderer.reset();
    }
  }

  @override
  void didUpdateWidget(CentralPanelTabView old) {
    super.didUpdateWidget(old);
    if (old.visible != widget.visible) {
      for (final renderer in renderers) {
        renderer.setVisible(widget.visible);
      }
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
    RootModel root = context.read<RootModel>();
    context.watch<SegmentModel>();
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
  final String? activeMode;
  const CentralPanel({super.key, required this.width, required this.activeMode});

  @override
  State<CentralPanel> createState() => _CentralPanelState();
}

class _CentralPanelState extends State<CentralPanel> {
  @override
  Widget build(BuildContext context) {
    debugPrint("_CentralPanelState build() activeMode=${widget.activeMode}");
    final isUserSteps = widget.activeMode == 'usersteps';
    final isSettings = widget.activeMode == 'settings';
    final index = isUserSteps ? 0 : isSettings ? 1 : 2;
    return IndexedStack(
      index: index,
      children: [
        SizedBox(
          width: widget.width,
          child: CentralPanelUserSteps(
            width: widget.width,
            visible: isUserSteps,
          ),
        ),
        SizedBox(
          width: widget.width,
          child: CentralPanelPDF(
            width: widget.width,
            visible: isSettings,
          ),
        ),
        SizedBox(
          width: widget.width,
          child: CentralPanelOverview(
            width: widget.width,
            visible: !isUserSteps && !isSettings,
          ),
        ),
      ],
    );
  }
}
