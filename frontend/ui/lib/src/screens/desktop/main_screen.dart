import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/screen_configuration.dart';
import 'package:ui/src/models/segmentmodel.dart';
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

class _MainScreenProviders extends StatelessWidget {
  final FutureRenderer futureRenderer;
  final Widget child;

  const _MainScreenProviders({
    required this.futureRenderer,
    required this.child,
  });

  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [ChangeNotifierProvider.value(value: futureRenderer)],
      child: child,
    );
  }
}

class MainScreen extends StatefulWidget {
  const MainScreen({super.key});

  @override
  State<MainScreen> createState() => _MainScreenState();
}

class _MainScreenState extends State<MainScreen> {
  FutureRenderer? futureRenderer;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (futureRenderer == null) {
      SegmentModel segmentModel = Provider.of<SegmentModel>(context);
      futureRenderer = FutureRenderer(
        bridge: segmentModel.backend,
        segment: segmentModel.segment,
        clients: [TrackData.map, TrackData.profile],
        kinds: allkinds(),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    assert(futureRenderer != null);
    return _MainScreenProviders(
      futureRenderer: futureRenderer!,
      child: _MainScaffold(),
    );
  }
}
