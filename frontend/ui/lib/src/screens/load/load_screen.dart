import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;
import 'package:ui/src/widgets/small.dart';
import 'package:ui/utils.dart';

import 'eventwidget.dart';
import 'model.dart';

class _GPXStrings {
  final LoadScreenModel screenModel;

  _GPXStrings({required this.screenModel});

  bridge.SegmentStatistics? statistics;
  void setData(bridge.SegmentStatistics s) {
    statistics = s;
  }

  String? km() {
    if (statistics == null) {
      return null;
    }
    double km = statistics!.distanceEnd / 1000;
    return "${km.toStringAsFixed(0)} km";
  }

  String? elevation() {
    if (statistics == null) {
      return null;
    }
    double e = statistics!.elevationGain;
    return "${e.toStringAsFixed(0)} m";
  }
}

class _GPXCard extends StatelessWidget {
  @override
  Widget build(BuildContext ctx) {
    LoadScreenModel model = Provider.of<LoadScreenModel>(ctx);
    _GPXStrings strings = _GPXStrings(screenModel: model);
    if (model.hasDone(Job.gpx)) {
      strings.setData(model.statistics());
    }
    Widget inner = Table(
      columnWidths: const {0: IntrinsicColumnWidth(), 1: FlexColumnWidth()},
      children: [
        TableRow(children: [SmallText(text: "Track"), SmallText(text: "")]),
        TableRow(
          children: [
            SmallText(text: "Length"),
            EventWidget(target: Job.gpx, forcedString: strings.km()),
          ],
        ),
        TableRow(
          children: [
            SmallText(text: "Elevation"),
            EventWidget(target: Job.gpx, forcedString: strings.elevation()),
          ],
        ),
      ],
    );

    return Card(elevation: 4, child: inner);
  }
}

class _ControlStrings {
  final LoadScreenModel screenModel;

  _ControlStrings({required this.screenModel});

  String? count() {
    if (!screenModel.hasDone(Job.controls)) {
      return null;
    }
    return "${screenModel.controlsCount()}";
  }
}

class ControlsCard extends StatelessWidget {
  const ControlsCard({super.key});

  @override
  Widget build(BuildContext ctx) {
    LoadScreenModel model = Provider.of<LoadScreenModel>(ctx);
    _ControlStrings strings = _ControlStrings(screenModel: model);
    Widget inner = Table(
      columnWidths: const {0: IntrinsicColumnWidth(), 1: FlexColumnWidth()},
      children: [
        TableRow(children: [SmallText(text: "Controls"), SmallText(text: "")]),
        TableRow(
          children: [
            SmallText(text: "Number"),
            EventWidget(target: Job.controls, forcedString: strings.count()),
          ],
        ),
      ],
    );
    return Card(elevation: 4, child: inner);
  }
}

class _OSMCard extends StatelessWidget {
  void onRetryPressed(LoadScreenModel model) {
    model.retry(Job.osm);
  }

  @override
  Widget build(BuildContext ctx) {
    developer.log("OSMCard build ");
    LoadScreenModel model = Provider.of<LoadScreenModel>(ctx);

    Widget row = EventWidget(target: Job.osm);
    if (model.error(Job.osm) != null) {
      row = Row(
        children: [
          EventWidget(target: Job.osm),
          ElevatedButton(
            onPressed: () => onRetryPressed(model),
            child: const Text("retry"),
          ),
        ],
      );
    }

    Widget inner = Table(
      columnWidths: const {0: IntrinsicColumnWidth(), 1: FlexColumnWidth()},
      children: [
        TableRow(children: [SmallText(text: "OSM"), SmallText(text: "")]),
        TableRow(children: [SmallText(text: "Status"), row]),
      ],
    );
    return Card(elevation: 4, child: inner);
  }
}

String _title(LoadScreenModel model) {
  if (model.doneAll()) {
    return "Loaded";
  }
  return "Loading...";
}

class _BodyWidget extends StatelessWidget {
  void onButtonPressed(BuildContext context) {
    try {
      Provider.of<SegmentModel>(context, listen: false);
      gotoOverview(context);
    } catch (e) {
      developer.log("[SegmentModel not yet available]");
    }
  }

  @override
  Widget build(BuildContext ctx) {
    LoadScreenModel model = Provider.of<LoadScreenModel>(ctx);
    Widget button = ElevatedButton(
      onPressed: null,
      child: Text("Please wait..."),
    );
    if (model.doneAll()) {
      button = ElevatedButton(
        onPressed: () => onButtonPressed(ctx),
        child: Text("OK"),
      );
    }
    Widget vspace = SizedBox(height: 20);
    return SmallCentralWidget(
      child: Column(
        children: [
          _GPXCard(),
          vspace,
          ControlsCard(),
          vspace,
          _OSMCard(),
          vspace,
          button,
        ],
      ),
    );
  }
}

class _LoadScaffold extends StatefulWidget {
  @override
  State<_LoadScaffold> createState() => _LoadScaffoldState();
}

class _LoadScaffoldState extends State<_LoadScaffold> {
  Widget buildScaffold(BuildContext ctx) {
    LoadScreenModel model = Provider.of<LoadScreenModel>(ctx);
    return Scaffold(
      appBar: AppBar(title: Text(_title(model))),
      body: _BodyWidget(),
    );
  }

  @override
  void didChangeDependencies() {
    // this start the jobs as soon as the screen is shown.
    super.didChangeDependencies();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      final model = context.read<LoadScreenModel>();
      if (model.needsStart()) {
        model.start();
      }
    });
  }

  @override
  Widget build(BuildContext ctx) {
    LoadScreenModel _ = Provider.of<LoadScreenModel>(ctx);
    debugPrint("LoadScreen build");
    return buildScaffold(ctx);
  }
}

class _LoadScreenProviders extends MultiProvider {
  final UserInput userInput;
  _LoadScreenProviders({required this.userInput, required Widget child})
    : super(
        providers: [
          ChangeNotifierProxyProvider2<RootModel, EventModel, LoadScreenModel>(
            create: (context) {
              EventModel events = Provider.of<EventModel>(
                context,
                listen: false,
              );
              developer.log("make LoadScreenModel");
              return LoadScreenModel(
                backend: getBackend(context),
                rootModel: Provider.of<RootModel>(context, listen: false),
                events: events,
                userInput: userInput,
              );
            },
            update: (context, root, event, loadscreen) {
              loadscreen!.onChanged(root, event);
              return loadscreen;
            },
          ),
        ],
        child: child,
      );
}

class LoadScreen extends StatelessWidget {
  final UserInput userInput;
  const LoadScreen({super.key, required this.userInput});

  @override
  Widget build(BuildContext context) {
    return _LoadScreenProviders(userInput: userInput, child: _LoadScaffold());
  }
}
