import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;
import 'package:ui/src/screens/load/parts_card.dart';
import 'package:ui/src/widgets/small.dart';
import 'package:ui/src/utils/utils.dart';

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
    Widget inner = Column(children: [SmallText(text: "Track")]);
    if (model.hasDone(Job.gpx)) {
      strings.setData(model.statistics());
      inner = Column(
        children: [
          SmallText(text: "Track"),
          EventWidget(target: Job.gpx, forcedString: strings.km()),
          EventWidget(target: Job.gpx, forcedString: strings.elevation()),
        ],
      );
    }
    Object? error = model.hasFailed(Job.gpx);
    if (error != null) {
      inner = Column(
        children: [
          SmallText(text: "Track"),
          SmallText(text: errorString(error)),
        ],
      );
    }

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
    Widget inner = Column(
      children: [
        SmallText(text: "Controls"),
        EventWidget(target: Job.controls, forcedString: strings.count()),
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
    if (model.hasFailed(Job.osm) != null) {
      row = Column(
        mainAxisAlignment: MainAxisAlignment.center,
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          EventWidget(target: Job.osm),
          ElevatedButton(
            onPressed: () => onRetryPressed(model),
            child: const Text("retry"),
          ),
        ],
      );
    }

    Widget inner = Column(children: [SmallText(text: "OSM"), row]);
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
  void onOKPressed(BuildContext context) {
    try {
      Provider.of<SegmentModel>(context, listen: false);
      gotoOverview(context);
    } catch (e) {
      developer.log("[SegmentModel not yet available]");
    }
  }

  void onHomePressed(BuildContext context) {
    gotoHome(context);
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
        onPressed: () => onOKPressed(ctx),
        child: Text("OK"),
      );
    }
    if (model.failed().isNotEmpty) {
      button = ElevatedButton(
        onPressed: () => onHomePressed(ctx),
        child: Text("Home"),
      );
    }
    Widget vspace = SizedBox(height: 20);
    return SmallCentralWidget(
      child: Column(
        children: [
          Flexible(flex: 1, child: PartsCard()),
          vspace,
          SizedBox(
            height: 140,
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Expanded(child: _GPXCard()),
                Expanded(child: ControlsCard()),
                Expanded(child: _OSMCard()),
              ],
            ),
          ),
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
              EventModel events = Provider.of(context, listen: false);
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
