import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/models/screen_configuration.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/routes.dart';
import 'package:wpx/src/screens/desktop/central_panel.dart';
import 'package:wpx/src/screens/desktop/side_panel.dart';
import 'package:wpx/src/widgets/editable_text.dart';

class MainScreen extends StatefulWidget {
  final GoRouterState routerState;
  const MainScreen({super.key, required this.routerState});

  @override
  State<MainScreen> createState() => _MainScreenState();
}

class _MainScreenState extends State<MainScreen> {
  String? _activeMode;

  @override
  void initState() {
    super.initState();
    _syncModeFromState(widget.routerState);
  }

  @override
  void didUpdateWidget(MainScreen oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.routerState != oldWidget.routerState) {
      _syncModeFromState(widget.routerState);
    }
  }

  void _syncModeFromState(GoRouterState state) {
    final mode = state.uri.queryParameters['mode'];
    if (mode != _activeMode) {
      setState(() => _activeMode = mode);
    }
  }

  void _onModeChanged(String? mode) {
    final target = mode != null ? '/overview?mode=$mode' : '/overview';
    context.go(target);
  }

  @override
  Widget build(BuildContext context) {
    ScreenConfiguration screen = context.watch<ScreenConfiguration>();
    Widget div = VerticalDivider(
      color: Colors.lightBlue,
      thickness: 1,
      width: 1,
    );

    debugPrint("[1] build MainScreen");
    assert(context.read<RootModel>().isLoaded());

    SegmentModel segmentModel = context.watch<SegmentModel>();
    String trackName = segmentModel.trackFileName();

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: Icon(Icons.home),
          onPressed: () => gotoHome(context),
        ),
        title: WritableText(
          initialName: trackName,
          onSubmitted: (newName) async {
            context.read<SegmentModel>().updateTrackfileName(newName: newName);
          },
        ),
      ),
      body: Row(
        children: [
          div,
          SidePanel(
            width: 480,
            activeMode: _activeMode,
            onModeChanged: _onModeChanged,
          ),
          div,
          CentralPanel(width: screen.width - 500, activeMode: _activeMode),
        ],
      ),
    );
  }
}
