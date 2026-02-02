import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/log.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/utils.dart';

class _SegmentsView extends StatefulWidget {
  @override
  State<_SegmentsView> createState() => _SegmentsViewState();
}

class _SegmentsViewState extends State<_SegmentsView> {
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

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        return const Text("building");
      },
    );
  }
}

class _SegmentsConsumer extends StatelessWidget {
  @override
  Widget build(BuildContext ctx) {
    return Center(
      child: Container(
        constraints: const BoxConstraints(maxWidth: 1500),
        child: Column(children: [Expanded(child: _SegmentsView())]),
      ),
    );
  }
}

class SegmentsScaffold extends StatelessWidget {
  const SegmentsScaffold({super.key});

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
    return Scaffold(appBar: appBar(ctx), body: _SegmentsConsumer());
  }
}
