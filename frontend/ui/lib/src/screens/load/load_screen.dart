import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/events.dart';
import 'package:wpx/src/models/kindsmodel.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/routes.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bridge;
import 'package:wpx/src/screens/load/parts_card.dart';
import 'package:wpx/src/widgets/small.dart';
import 'package:wpx/src/utils/utils.dart';

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
    LoadScreenModel model = ctx.watch<LoadScreenModel>();
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

  String? controlsCount() {
    // Note: this `if` is important. Otherwise: crash in backend,
    // since the backend_data may be none.
    if (!screenModel.hasDone(Job.controls)) {
      return null;
    }
    return "${screenModel.controlsCount()} controls";
  }

  String? waypointsCount() {
    // Note: this `if` is important. Otherwise: crash in backend,
    // since the backend_data may be none.
    if (!screenModel.hasDone(Job.controls)) {
      return null;
    }
    return "${screenModel.waypointsCount()} waypoints";
  }
}

class ControlsCard extends StatelessWidget {
  const ControlsCard({super.key});

  @override
  Widget build(BuildContext ctx) {
    LoadScreenModel model = ctx.watch<LoadScreenModel>();
    _ControlStrings strings = _ControlStrings(screenModel: model);
    Widget inner = Column(
      children: [
        SmallText(text: "Points"),
        EventWidget(target: Job.gpx, forcedString: strings.waypointsCount()),
        EventWidget(
          target: Job.controls,
          forcedString: strings.controlsCount(),
        ),
      ],
    );
    return Card(elevation: 4, child: inner);
  }
}

class _OSMCard extends StatelessWidget {
  void onSkipPressed(LoadScreenModel model) {
    model.cancelOsm();
  }

  @override
  Widget build(BuildContext ctx) {
    debugPrint("OSMCard build ");
    LoadScreenModel model = ctx.watch<LoadScreenModel>();

    Widget row = OsmEventWidget(target: Job.osm);
    if (model.hasFailed(Job.osm) != null || model.runningJob() == Job.osm) {
      EdgeInsets valuePadding = const EdgeInsets.fromLTRB(15, 0, 15, 0);
      Widget eventWidget = OsmEventWidget(target: Job.osm);
      if (model.hasFailed(Job.osm) != null) {
        debugPrint("failure");
        eventWidget = EventWidget(target: Job.osm);
      }
      row = Column(
        mainAxisAlignment: MainAxisAlignment.start,
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          eventWidget,
          SizedBox(height: 5),
          OutlinedButton(
            onPressed:
                model.runningJob() == Job.osm
                    ? () => onSkipPressed(model)
                    : null,
            style: ElevatedButton.styleFrom(
              padding: valuePadding,
              minimumSize: Size.zero,
              tapTargetSize: MaterialTapTargetSize.shrinkWrap,
            ),
            child: Text("cancel", textAlign: TextAlign.center),
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

class _BodyWidget extends StatefulWidget {
  @override
  State<_BodyWidget> createState() => _BodyWidgetState();
}

class _BodyWidgetState extends State<_BodyWidget> {
  void onOKPressed(BuildContext context) async {
    try {
      context.read<SegmentModel>();
    } catch (e) {
      developer.log("[SegmentModel not yet available]");
    }
    RootModel root = context.read<RootModel>();
    assert(root.isLoaded());
    bridge.TrackFile trackFile =
        await context.read<SegmentModel>().createTrackFile();
    assert(root.isLoaded());
    debugPrint("create: set user input ${trackFile.name}");
    assert(root.isLoaded());
    root.setTrackFile(trackFile);
    assert(root.isLoaded());
    root.notify();
    assert(root.isLoaded());
    if (context.mounted) {
      gotoOverview(context);
    }
  }

  void onHomePressed(BuildContext context) {
    gotoHome(context);
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    LoadScreenModel load = context.watch();
    KindsModel kinds = context.watch();
    RootModel root = context.watch();
    if (root.isLoaded()) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        // KindsModel needs the statistics to decide what
        // what kinds are disabled or not.
        kinds.updateStatistics(load.statistics());
      });
    }
    kinds.osmIsLoaded = load.hasFailed(Job.osm) == null;
  }

  @override
  Widget build(BuildContext ctx) {
    LoadScreenModel model = context.watch<LoadScreenModel>();

    bool okAllowed =
        model.hasDone(Job.controls) && model.runningJob() == Job.none;

    Widget button = ElevatedButton(
      onPressed: null,
      child: Text("Please wait..."),
    );
    if (okAllowed) {
      button = ElevatedButton(
        onPressed: () => onOKPressed(ctx),
        child: Text("OK"),
      );
    } else if (model.runningJob() == Job.none) {
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
    LoadScreenModel model = context.watch<LoadScreenModel>();
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
    LoadScreenModel _ = context.watch<LoadScreenModel>();
    debugPrint("LoadScreen build");
    return buildScaffold(ctx);
  }
}

class _LoadScreenProviders extends MultiProvider {
  final PendingContent userInput;
  _LoadScreenProviders({required this.userInput, required Widget child})
    : super(
        providers: [
          ChangeNotifierProxyProvider2<RootModel, EventModel, LoadScreenModel>(
            create: (context) {
              EventModel events = context.read();
              developer.log("make LoadScreenModel");
              return LoadScreenModel(
                backend: getBackend(context),
                rootModel: context.read<RootModel>(),
                events: events,
                userInput: userInput,
              );
            },
            update: (context, root, event, loadscreen) {
              // does not pass all events
              loadscreen!.onChanged(root, event);
              return loadscreen;
            },
          ),
        ],
        child: child,
      );
}

class LoadScreen extends StatelessWidget {
  final PendingContent userInput;
  const LoadScreen({super.key, required this.userInput});

  @override
  Widget build(BuildContext context) {
    return _LoadScreenProviders(userInput: userInput, child: _LoadScaffold());
  }
}
