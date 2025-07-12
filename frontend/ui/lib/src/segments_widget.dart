import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/segment_stack.dart';

class RenderingsProvider extends MultiProvider {
  final Renderers renderers;

  RenderingsProvider(Renderers r, Widget child, {super.key})
    : renderers = r,
      super(
        providers: [
          ChangeNotifierProvider.value(value: r.trackRendering),
          ChangeNotifierProvider.value(value: r.waypointsRendering),
        ],
        child: child,
      );
}

class SegmentsView extends StatelessWidget {
  final SegmentsProvider? segmentsProvider;
  const SegmentsView({super.key, this.segmentsProvider});

  List<RenderingsProvider> renderingProviders(
    SegmentsProvider segmentsProvider,
  ) {
    List<RenderingsProvider> ret = [];
    developer.log("[_initRenderingProviders]");
    var S = segmentsProvider.segments();
    developer.log("[S]=${S.length}");
    assert(ret.isEmpty);
    for (var renderer in S) {
      var w = RenderingsProvider(renderer, SegmentStack());
      w.renderers.trackRendering.start();
      ret.add(w);
    }
    developer.log("[renderingProviders] [build] #segments=${ret.length}");
    return ret;
  }

  @override
  Widget build(BuildContext context) {
    var segments = renderingProviders(segmentsProvider!);
    developer.log("[segments] [build] #segments=${segments.length}");
    List<Tab> tabs = [];
    for (var s in segments) {
      var id = s.renderers.trackRendering.segment.id();
      tabs.add(Tab(text: "segment ${id.toInt()}"));
    }
    return DefaultTabController(
      length: segments.length,
      child: Scaffold(
        appBar: TabBar(tabs: tabs),
        body: TabBarView(children: segments),
      ),
    );
  }
}

class SegmentsConsumer extends StatelessWidget {
  const SegmentsConsumer({super.key});
  @override
  Widget build(BuildContext ctx) {
    return Consumer<SegmentsProvider>(
      builder: (context, segmentsProvider, child) {
        developer.log(
          "[SegmentsConsumer] length=${segmentsProvider.segments().length}",
        );
        return Center(
          child: Container(
            constraints: const BoxConstraints(maxWidth: 1500),
            child: Column(
              children: [
                Expanded(
                  /*child: Text(
                    "gpx has ${segmentsProvider.segments().length} segments",
                  ),*/
                  child: SegmentsView(segmentsProvider: segmentsProvider),
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}

class SegmentsProviderWidget extends StatelessWidget {
  const SegmentsProviderWidget({super.key});

  Widget wait() {
    return Scaffold(
      appBar: AppBar(title: const Text('Segments')),
      body: Center(child: Column(children: [Text("loading...")])),
    );
  }

  @override
  Widget build(BuildContext ctx) {
    return Consumer<RootModel>(
      builder: (context, rootModel, child) {
        if (rootModel.provider() == null) {
          return wait();
        }
        developer.log(
          "[SegmentsProviderWidget] ${rootModel.provider()?.filename()} length=${rootModel.provider()?.segments().length}",
        );
        return ChangeNotifierProvider.value(
          value: rootModel.provider(),
          builder: (context, child) {
            return Scaffold(
              appBar: AppBar(title: const Text('Segments')),
              body: SegmentsConsumer(),
            );
          },
        );
      },
    );
  }
}
