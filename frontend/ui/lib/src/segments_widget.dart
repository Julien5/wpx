import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/futurerenderer.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/segment_stack.dart';

class RenderingsProvider extends MultiProvider {
  final Renderers renderers;

  RenderingsProvider(Renderers r, Widget child, {super.key})
    : renderers = r,
      super(
        providers: [
          ChangeNotifierProvider.value(value: r.profileRendering),
          ChangeNotifierProvider.value(value: r.mapRendering),
          ChangeNotifierProvider.value(value: r.yaxisRendering),
        ],
        child: child,
      );
}

class SegmentsView extends StatelessWidget {
  const SegmentsView({super.key});

  List<RenderingsProvider> renderingProviders(RootModel rootModel) {
    List<RenderingsProvider> ret = [];
    developer.log("[_initRenderingProviders]");
    rootModel.updateSegments();
    var S = rootModel.segments();
    developer.log("[S]=${S.length}");
    assert(ret.isEmpty);
    for (var segment in S.keys) {
      var data = S[segment]!;
      var w = RenderingsProvider(data.renderers, SegmentView());
      ret.add(w);
    }
    developer.log("[renderingProviders] [build] #segments=${ret.length}");
    return ret;
  }

  @override
  Widget build(BuildContext context) {
    var rootModel = Provider.of<RootModel>(context);
    var segments = renderingProviders(rootModel);
    developer.log("[segments] [build] #segments=${segments.length}");
    List<Tab> tabs = [];
    for (var s in segments) {
      var id = s.renderers.profileRendering.segment.id();
      tabs.add(Tab(text: "Page ${1 + id.toInt()}"));
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
    return Center(
      child: Container(
        constraints: const BoxConstraints(maxWidth: 1500),
        child: Column(
          children: [
            Expanded(
              /*child: Text(
                    "gpx has ${segmentsProvider.segments().length} segments",
                  ),*/
              child: SegmentsView(),
            ),
          ],
        ),
      ),
    );
  }
}

class SegmentsScaffold extends StatelessWidget {
  const SegmentsScaffold({super.key});

  Widget exportButton(BuildContext context) {
    return ElevatedButton(
      child: const Text('GPX/PDF export'),
      onPressed: () {
        Navigator.of(context).pushNamed(RouteManager.exportView);
      },
    );
  }

  @override
  Widget build(BuildContext ctx) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Preview'),
        actions: <Widget>[exportButton(ctx)],
      ),
      body: SegmentsConsumer(),
    );
  }
}
