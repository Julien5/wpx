//import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/future_rendering_widget.dart';

class InteractiveMapView extends StatelessWidget {
  const InteractiveMapView({super.key});

  @override
  Widget build(BuildContext context) {
    return Consumer<MapRenderer>(
      builder: (context, mapRenderer, child) {
        return LayoutBuilder(
          builder: (BuildContext context, BoxConstraints constraints) {
            mapRenderer.setSize(constraints.biggest);
            return FutureRenderingWidget(interactive: true);
          },
        );
      },
    );
  }
}

class InteractiveConsumer extends StatelessWidget {
  const InteractiveConsumer({super.key});
  @override
  Widget build(BuildContext ctx) {
    return Center(
      child: Container(
        constraints: const BoxConstraints(maxWidth: 1500),
        child: Column(children: [Expanded(child: InteractiveMapView())]),
      ),
    );
  }
}

class InteractiveScreen extends StatefulWidget {
  const InteractiveScreen({super.key});

  @override
  State<InteractiveScreen> createState() => _InteractiveScreenState();
}

class _InteractiveScreenState extends State<InteractiveScreen> {
  Segment? trackSegment;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    RootModel root = Provider.of<RootModel>(context, listen: false);
    // This sets tracksSegment only if it is null.
    trackSegment ??= root.trackSegment();
  }

  AppBar? appBar(BuildContext ctx) {
    return AppBar(
      title: const Text('Map'),
      actions: <Widget>[
        ElevatedButton(
          child: const Text('Settings'),
          onPressed: () {
            Navigator.of(ctx).pushNamed(RouteManager.settingsView);
          },
        ),
      ],
    );
  }

  /* 
  * This widget gets rebuilt even when it is not visible.
  * This is intended:
  * https://github.com/flutter/flutter/issues/11655
  * (1) we should not build trackSegment in the build() method
  *     => moved to didChangeDependencies (like initState but with context)
  * (2) didChangeDependencies is called multiple times too, probable because
  *     of setParamets and notifyListeners (not sure). To work around this,
  *     we update the trackSegment only if it is not null.
  */
  @override
  Widget build(BuildContext ctx) {
    RootModel root = Provider.of<RootModel>(ctx);
    if (trackSegment == null) {
      return Text("building...");
    }
    return Scaffold(
      appBar: appBar(ctx),
      body: ChangeNotifierProvider<MapRenderer>(
        create: (_) => MapRenderer(root.getBridge(), trackSegment!, allkinds()),
        child: InteractiveConsumer(),
      ),
    );
  }
}
