import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/log.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/waypointsmodel.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/utils.dart';
import 'segment_stack.dart';

class RenderersProvider extends MultiProvider {
  final Renderers renderers;

  RenderersProvider(Renderers r, WaypointsModel d, Widget child, {super.key})
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

class SegmentsView extends StatefulWidget {
  const SegmentsView({super.key});

  @override
  State<SegmentsView> createState() => _SegmentsViewState();
}

class _SegmentsViewState extends State<SegmentsView> {
  List<Segment>? segments;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (segments == null) {
      var rootModel = Provider.of<RootModel>(context);
      log("update segments");
      segments = rootModel.segments();
    }
  }

  List<RenderersProvider> renderingProviders(
    RootModel rootModel,
    ScreenOrientation screenOrientation,
  ) {
    List<RenderersProvider> ret = [];
    log("[_initRenderingProviders] length=${segments!.length}");
    for (var segment in segments!) {
      var w = RenderersProvider(
        Renderers.make(rootModel.getBridge(), segment, allkinds()),
        WaypointsModel(brd: rootModel.getBridge(), segment: segment),
        SegmentView(screenOrientation: screenOrientation),
      );
      ret.add(w);
    }
    log("[renderingProviders] [build] #segments=${ret.length}");
    return ret;
  }

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        return const Text("building");
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
