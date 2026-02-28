import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/screen_configuration.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/models/trackviewswitch.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/screens/desktop/central_panel.dart';
import 'package:ui/src/screens/desktop/side_panel.dart';

class _MainScaffold extends StatelessWidget {
  @override
  Widget build(BuildContext ctx) {
    ScreenConfiguration screen = Provider.of<ScreenConfiguration>(ctx);
    Widget div = VerticalDivider(
      color: Colors.lightBlue,
      thickness: 1,
      width: 1, // This is the horizontal space the widget occupies
    );
    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: Icon(Icons.home),
          onPressed: () => gotoHome(ctx),
        ),
        title: const Text('Overview'),
      ),
      body: Row(
        children: [
          div,
          SidePanel(width: 480),
          div,
          CentralPanel(width: screen.width - 500),
        ],
      ),
    );
  }
}

class _MainScreenProviders extends MultiProvider {
  final SegmentModel segmentModel;
  final List<TrackData> trackData;
  final Kinds kinds;
  _MainScreenProviders({
    required this.segmentModel,
    required this.trackData,
    required this.kinds,
    required Widget child,
  }) : super(
         providers: [
           ChangeNotifierProvider(
             create: (_) => TrackViewsSwitch(exposed: TrackViewsSwitch.wmp()),
           ),
           ChangeNotifierProvider(
             create:
                 (_) => FutureRenderer(
                   bridge: segmentModel.backend,
                   segment: segmentModel.segment,
                   trackData: trackData,
                   kinds: kinds,
                 ),
           ),
         ],
         child: child,
       );
}

class MainScreen extends StatelessWidget {
  const MainScreen({super.key});

  @override
  Widget build(BuildContext context) {
    SegmentModel segmentModel = Provider.of<SegmentModel>(context);
    context.watch<SegmentModel>();
    return _MainScreenProviders(
      segmentModel: segmentModel,
      trackData: [TrackData.map, TrackData.profile],
      kinds: allkinds(),
      child: _MainScaffold(),
    );
  }
}
