//import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/futurerenderer.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/widgets/future_rendering_widget.dart';
import 'package:wpx/src/utils/utils.dart';

class _InteractiveMapView extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Consumer<FutureRenderer>(
      builder: (context, mapRenderer, child) {
        return LayoutBuilder(
          builder: (BuildContext context, BoxConstraints constraints) {
            mapRenderer.setSize(TrackData.map, constraints.biggest);
            return FutureRenderingWidget(
              trackData: TrackData.map,
              interactive: true,
            );
          },
        );
      },
    );
  }
}

class _InteractiveConsumer extends StatelessWidget {
  @override
  Widget build(BuildContext ctx) {
    return Center(
      child: Container(
        constraints: const BoxConstraints(maxWidth: 1500),
        child: Column(children: [Expanded(child: _InteractiveMapView())]),
      ),
    );
  }
}

class InteractiveScaffold extends StatefulWidget {
  const InteractiveScaffold({super.key});

  @override
  State<InteractiveScaffold> createState() => _InteractiveScaffoldState();
}

class _InteractiveScaffoldState extends State<InteractiveScaffold> {
  AppBar? appBar(BuildContext ctx) {
    return AppBar(
      title: const Text('Map'),
      actions: <Widget>[
        ElevatedButton(
          child: const Text('Settings'),
          onPressed: () {
            //Navigator.of(ctx).pushNamed(RouteManager.settingsView);
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
    SegmentModel track = Provider.of<SegmentModel>(ctx);
    Bridge backend = getBackend(ctx);
    return Scaffold(
      appBar: appBar(ctx),
      body: ChangeNotifierProvider<FutureRenderer>(
        create:
            (_) => FutureRenderer(
              bridge: backend,
              segment: track.segment,
              kinds: allkinds(),
              clients: [TrackData.map],
            ),
        child: _InteractiveConsumer(),
      ),
    );
  }
}
