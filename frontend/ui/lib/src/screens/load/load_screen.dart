import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;
import 'package:ui/src/screens/home/home_screen.dart';
import 'package:ui/src/widgets/small.dart';
import 'package:visibility_detector/visibility_detector.dart';

class EventWidget extends StatefulWidget {
  final LoadScreenModel screenModel;
  final Job target;
  final String? forcedString;
  const EventWidget({
    super.key,
    required this.screenModel,
    required this.target,
    this.forcedString,
  });

  @override
  State<EventWidget> createState() => _EventWidgetState();
}

class _EventWidgetState extends State<EventWidget> {
  EventModel? model;

  @override
  Widget build(BuildContext context) {
    if (widget.forcedString != null) {
      return SmallText(text: widget.forcedString!);
    }
    EventModel model = Provider.of<EventModel>(context, listen: false);
    return StreamBuilder<String>(
      stream: model.broadcastStream,
      builder: (context, snap) {
        final error = snap.error;
        String event = "....";
        if (error != null) {
          event = error.toString();
          developer.log("error: ${error.toString()}");
        }
        final data = snap.data;
        if (data != null) {
          event = data;
        }
        return SmallText(
          text: filterEvent(event, widget.target, widget.screenModel),
        );
      },
    );
  }
}

String safeLast(String? event) {
  if (event == null) {
    return "...";
  }
  //developer.log("====> ${event.events.last} ($n)");
  return event;
}

String errorString(Error e) {
  if (e is bridge.Error_MissingElevation) {
    //var index = e.index;
    return "The track misses elevation data.";
  }
  if (e is bridge.Error_GPXHasNoSegment) {
    return "no segment in gpx";
  }
  if (e is bridge.Error_GPXInvalid) {
    return "invalid gpx file";
  }
  if (e is bridge.Error_OSMDownloadFailed) {
    return "download failed";
  }
  return "";
}

String filterEvent(String? event, Job targetJob, LoadScreenModel screenModel) {
  if (screenModel.errors.containsKey(targetJob)) {
    //return "error: [${errorString(screenModel.errors[targetJob]!)}]";
    return errorString(screenModel.errors[targetJob]!);
  }
  if (screenModel.running != null && screenModel.running! == targetJob) {
    //return "event: [${safeLast(eventModel)}]";
    return safeLast(event);
  }
  if (screenModel.hasDone(targetJob)) {
    return "done";
  }
  return "..";
}

class GPXStrings {
  final LoadScreenModel screenModel;

  GPXStrings({required this.screenModel});

  SegmentStatistics? statistics;
  void setData(SegmentStatistics s) {
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

class GPXCard extends StatelessWidget {
  const GPXCard({super.key});

  @override
  Widget build(BuildContext ctx) {
    LoadScreenModel model = Provider.of<LoadScreenModel>(ctx);
    GPXStrings strings = GPXStrings(screenModel: model);
    if (model.hasDone(Job.gpx)) {
      strings.setData(model.statistics()!);
    }
    Widget inner = Table(
      columnWidths: const {0: IntrinsicColumnWidth(), 1: FlexColumnWidth()},
      children: [
        TableRow(children: [SmallText(text: "Track"), SmallText(text: "")]),
        TableRow(
          children: [
            SmallText(text: "Length"),
            EventWidget(
              screenModel: model,
              target: Job.gpx,
              forcedString: strings.km(),
            ),
          ],
        ),
        TableRow(
          children: [
            SmallText(text: "Elevation"),
            EventWidget(
              screenModel: model,
              target: Job.gpx,
              forcedString: strings.elevation(),
            ),
          ],
        ),
      ],
    );

    return Card(elevation: 4, child: inner);
  }
}

class ControlStrings {
  final LoadScreenModel screenModel;

  ControlStrings({required this.screenModel});

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
    ControlStrings strings = ControlStrings(screenModel: model);
    Widget inner = Table(
      columnWidths: const {0: IntrinsicColumnWidth(), 1: FlexColumnWidth()},
      children: [
        TableRow(children: [SmallText(text: "Controls"), SmallText(text: "")]),
        TableRow(
          children: [
            SmallText(text: "Number"),
            EventWidget(
              screenModel: model,
              target: Job.controls,
              forcedString: strings.count(),
            ),
          ],
        ),
      ],
    );
    return Card(elevation: 4, child: inner);
  }
}

class OSMCard extends StatelessWidget {
  const OSMCard({super.key});

  @override
  Widget build(BuildContext ctx) {
    developer.log("OSMCard build ");
    LoadScreenModel model = Provider.of<LoadScreenModel>(ctx);
    Widget inner = Table(
      columnWidths: const {0: IntrinsicColumnWidth(), 1: FlexColumnWidth()},
      children: [
        TableRow(children: [SmallText(text: "OSM"), SmallText(text: "")]),
        TableRow(
          children: [
            SmallText(text: "Status"),
            EventWidget(screenModel: model, target: Job.osm),
          ],
        ),
      ],
    );
    return Card(elevation: 4, child: inner);
  }
}

String title(LoadScreenModel model) {
  if (model.doneAll()) {
    return "Done";
  }
  return "Loading...";
}

class LoadScreen extends StatelessWidget {
  const LoadScreen({super.key});

  void gotoWheel(BuildContext context) {
    Navigator.of(context).pushNamed(RouteManager.wheelView);
  }

  Widget buildScaffold(BuildContext ctx) {
    LoadScreenModel model = Provider.of<LoadScreenModel>(ctx);
    return Scaffold(
      appBar: AppBar(title: Text(title(model))),
      body: Center(
        child: Column(
          children: [
            GPXCard(),
            ControlsCard(),
            OSMCard(),
            ElevatedButton(
              onPressed: () => {gotoWheel(ctx)},
              child: Text("OK"),
            ),
          ],
        ),
      ),
    );
  }

  void onVisibilityChanged(
    BuildContext context,
    VisibilityInfo visibilityInfo,
  ) {
    if (!context.mounted) {
      return;
    }
    var visiblePercentage = visibilityInfo.visibleFraction * 100;
    if (visiblePercentage == 100) {
      LoadScreenModel model = Provider.of<LoadScreenModel>(
        context,
        listen: false,
      );
      // The screen is now 100% visible and the transition is done
      if (model.needsStart()) {
        model.start();
      }
    }
  }

  @override
  Widget build(BuildContext ctx) {
    Provider.of<LoadScreenModel>(ctx);
    return VisibilityDetector(
      key: Key('LoadingScreen'),
      onVisibilityChanged: (info) => onVisibilityChanged(ctx, info),
      child: buildScaffold(ctx),
    );
  }
}

enum Job { gpx, osm, controls, none }

class FutureJob {
  final Future<void> future;
  final Job job;

  FutureJob({required this.future, required this.job});
}

class LoadScreenModel extends ChangeNotifier {
  Set<Job> done = {};
  Map<Job, Error> errors = {};
  Job? running;
  final RootModel root;
  final EventModel events;
  final UserInput userInput;
  FutureJob? runningFuture;
  LoadScreenModel({
    required this.root,
    required this.events,
    required this.userInput,
  });

  bool needsStart() {
    return running == null && done.isEmpty;
  }

  bool hasDone(Job job) {
    return done.contains(job);
  }

  static Job next(Job old) {
    if (old == Job.gpx) {
      return Job.controls;
    }
    if (old == Job.controls) {
      return Job.osm;
    }
    return Job.none;
  }

  void _makeFuture(Job job) {
    Future<void>? future;
    if (job == Job.gpx) {
      if (userInput.demo) {
        future = root.loadDemo();
      } else {
        assert(userInput.bytes != null);
        future = root.loadContent(userInput.bytes!);
      }
    } else if (job == Job.osm) {
      future = root.getBridge().loadOsm();
    } else if (job == Job.controls) {
      future = root.getBridge().loadControls(source: ControlSource.waypoints);
    } else {
      assert(false);
    }
    future!.then((_) => onCompleted(job)).catchError((error) {
      onError(job, error);
    });
    runningFuture = FutureJob(future: future, job: job);
  }

  void makeFuture(Job job) {
    running = job;
    SchedulerBinding.instance.addPostFrameCallback((_) {
      _makeFuture(job);
    });
  }

  void start() {
    startJob(Job.gpx);
  }

  void startJob(Job job) {
    events.take();
    developer.log("start $job");
    makeFuture(job);
    developer.log("future created");
    notifyListeners();
  }

  void onCompleted(Job job) {
    running = null;
    done.add(job);
    developer.log("notify");
    notifyListeners();
    Job nextJob = next(job);
    if (nextJob != Job.none) {
      Future.delayed(const Duration(milliseconds: 250), () {
        startJob(nextJob);
      });
    }
  }

  bool doneAll() {
    return done.contains(Job.gpx) &&
        done.contains(Job.controls) &&
        done.contains(Job.osm);
  }

  void onError(Job job, Error e) {
    developer.log("error: $e");
    errors[job] = e;
    notifyListeners();
  }

  SegmentStatistics? statistics() {
    if (!root.getBridge().isLoaded()) {
      developer.log("bridge not loaded");
      return null;
    }
    developer.log("bridge loaded");
    return root.statistics();
  }

  int controlsCount() {
    List<Waypoint> w = root.getBridge().getWaypoints(
      segment: root.trackSegment(),
      kinds: {InputType.control},
    );
    return w.length;
  }

  void onChange(RootModel root, EventModel event) {
    developer.log("LoadScreenModel::onChange");
    notifyListeners();
  }
}

class LoadScreenProviders extends MultiProvider {
  final UserInput userInput;
  LoadScreenProviders({
    super.key,
    required RootModel root,
    required this.userInput,
    required Widget child,
  }) : super(
         providers: [
           ChangeNotifierProvider.value(value: root),
           ChangeNotifierProvider.value(value: root.eventModel()),
           ChangeNotifierProxyProvider2<RootModel, EventModel, LoadScreenModel>(
             create: (context) {
               RootModel root = Provider.of<RootModel>(context, listen: false);
               EventModel events = Provider.of<EventModel>(
                 context,
                 listen: false,
               );
               return LoadScreenModel(
                 root: root,
                 events: events,
                 userInput: userInput,
               );
             },
             update: (context, root, events, loadscreen) {
               // the problem with this function is that we cannot distinguish
               // between changes in EventModel and RootModel
               loadscreen!.onChange(root, events);
               return loadscreen;
             },
           ),
         ],
         child: child,
       );
}

class LoadProvider extends StatelessWidget {
  final UserInput userInput;
  const LoadProvider({super.key, required this.userInput});

  @override
  Widget build(BuildContext context) {
    RootModel root = Provider.of<RootModel>(context);
    return LoadScreenProviders(
      root: root,
      userInput: userInput,
      child: LoadScreen(),
    );
  }
}
