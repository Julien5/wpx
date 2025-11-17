import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/waypointstable.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/utils.dart';
import 'segment_stack.dart';

class RenderersProvider extends MultiProvider {
  final Renderers renderers;

  RenderersProvider(Renderers r, WaypointsTableData d, Widget child, {super.key})
    : renderers = r,
      super(
        providers: [
          ChangeNotifierProvider.value(value: r.profileRenderer),
          ChangeNotifierProvider.value(value: r.mapRenderer),
          ChangeNotifierProvider.value(value: r.yaxisRenderer),
          ChangeNotifierProvider.value(value: d),
        ],
        child: child,
      );
}

class SegmentsView extends StatelessWidget {
  const SegmentsView({super.key});

  List<RenderersProvider> renderingProviders(
    RootModel rootModel,
    ScreenOrientation screenOrientation,
  ) {
    List<RenderersProvider> ret = [];
    developer.log("[_initRenderingProviders]");
    for (var segment in rootModel.segments()) {
      var w = RenderersProvider(
        Renderers.make(rootModel.getBridge(),segment),
        WaypointsTableData(brd: rootModel.getBridge(),segment: segment),
        SegmentView(screenOrientation: screenOrientation),
      );
      ret.add(w);
    }
    developer.log("[renderingProviders] [build] #segments=${ret.length}");
    return ret;
  }

  @override
  Widget build(BuildContext context) {
    var rootModel = Provider.of<RootModel>(context);

    return LayoutBuilder(
      builder: (context, constraints) {
        ScreenOrientation viewType = screenOrientation(
          Size(constraints.maxWidth, constraints.maxHeight),
        );

        var segments = renderingProviders(rootModel, viewType);
        developer.log("[segments] [build] #segments=${segments.length}");
        List<Tab> tabs = [];
        for (var s in segments) {
          var id = s.renderers.profileRenderer.segment.id();
          tabs.add(Tab(text: "Page ${1 + id.toInt()}"));
        }
        if (viewType == ScreenOrientation.desktop) {
          return DefaultTabController(
            length: segments.length,
            child: Scaffold(
              appBar: TabBar(tabs: tabs),
              body: TabBarView(children: segments),
            ),
          );
        }
        return DefaultTabController(
          length: segments.length,
          child: Column(
            children: [
              Expanded(child: TabBarView(children: segments)),
              const TabPageSelector(),
            ],
          ),
        );
      },
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
        child: Column(children: [Expanded(child: SegmentsView())]),
      ),
    );
  }
}

class SegmentsScreen extends StatelessWidget {
  const SegmentsScreen({super.key});

  AppBar? appBar(BuildContext ctx) {
    ScreenOrientation type = screenOrientation(MediaQuery.of(ctx).size);
    if (type == ScreenOrientation.landscape) {
      return null;
    }
    return AppBar(
      title: const Text('Preview'),
      actions: <Widget>[
        ElevatedButton(
          child: const Text('GPX/PDF export'),
          onPressed: () {
            Navigator.of(ctx).pushNamed(RouteManager.exportView);
          },
        ),
      ],
    );
  }

  @override
  Widget build(BuildContext ctx) {
    return Scaffold(appBar: appBar(ctx), body: SegmentsConsumer());
  }
}
