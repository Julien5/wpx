import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/screens/home/home_screen.dart';
import 'package:ui/src/widgets/small.dart';
import 'package:visibility_detector/visibility_detector.dart';

class StatisticsStrings {
  final SegmentStatistics statistics;

  StatisticsStrings({required this.statistics});
  String km() {
    double km = statistics.distanceEnd / 1000;
    return "${km.toStringAsFixed(0)} km";
  }

  String elevation() {
    double e = statistics.elevationGain;
    return "${e.toStringAsFixed(0)} m";
  }
}

class StreamWidget extends StatefulWidget {
  const StreamWidget({super.key});

  @override
  State<StreamWidget> createState() => _StreamWidgetState();
}

class _StreamWidgetState extends State<StreamWidget> {
  EventModel? model;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      try {
        setState(() {
          model = Provider.of<RootModel>(context, listen: false).eventModel();
        });
      } catch (e) {
        developer.log("Error: RootModel not found in context. Exception: $e");
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    if (model == null) {
      return Text("loading..");
    }
    return StreamBuilder<String>(
      stream: model!.stream,
      builder: (context, snap) {
        final error = snap.error;
        String text = "<null>";
        if (error != null) {
          text = error.toString();
          developer.log("error: ${error.toString()}");
        }
        final data = snap.data;
        if (data != null) {
          text = data;
        }
        return Text('text=$text');
      },
    );
  }
}

class GPXCard extends StatelessWidget {
  const GPXCard({super.key});

  @override
  Widget build(BuildContext ctx) {
    LoadScreenModel model = Provider.of<LoadScreenModel>(ctx);
    SegmentStatistics? statistics = model.statistics();
    Widget inner = Text("loading");
    if (statistics != null) {
      StatisticsStrings strings = StatisticsStrings(statistics: statistics);
      inner = Table(
        columnWidths: const {0: IntrinsicColumnWidth(), 1: FlexColumnWidth()},
        children: [
          TableRow(children: [SmallText(text: "GPX"), SmallText(text: "")]),
          TableRow(
            children: [
              SmallText(text: "Length"),
              SmallText(text: strings.km()),
            ],
          ),
          TableRow(
            children: [
              SmallText(text: "Elevation"),
              SmallText(text: strings.elevation()),
            ],
          ),
        ],
      );
    }
    return Card(elevation: 4, child: inner);
  }
}

class ControlsCard extends StatelessWidget {
  const ControlsCard({super.key});

  @override
  Widget build(BuildContext ctx) {
    LoadScreenModel model = Provider.of<LoadScreenModel>(ctx);
    Widget inner = Table(
      columnWidths: const {0: IntrinsicColumnWidth(), 1: FlexColumnWidth()},
      children: [
        TableRow(children: [SmallText(text: "Controls"), SmallText(text: "")]),
        TableRow(
          children: [
            SmallText(text: "Number"),
            SmallText(text: "${model.controlsCount()}"),
          ],
        ),
      ],
    );
    return Card(elevation: 4, child: inner);
  }
}

class LoadScreen extends StatelessWidget {
  const LoadScreen({super.key});

  void gotoWheel(BuildContext context) {
    Navigator.of(context).pushNamed(RouteManager.wheelView);
  }

  Widget buildScaffold(BuildContext ctx) {
    LoadScreenModel model = Provider.of<LoadScreenModel>(ctx);
    return Scaffold(
      appBar: AppBar(title: Text('Loading...${model.running}')),
      body: Center(
        child: Column(
          children: [
            GPXCard(),
            if (model.hasDone(Job.controls)) ControlsCard(),
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
      return Job.osm;
    }
    if (old == Job.osm) {
      return Job.controls;
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
    developer.log("start $job");
    makeFuture(job);
    developer.log("future created");
    notifyListeners();
  }

  void onCompleted(Job job) {
    done.add(job);
    developer.log("notify");
    notifyListeners();
    Job nextJob = next(job);
    if (nextJob != Job.none) {
      Future.delayed(const Duration(milliseconds: 25), () {
        startJob(nextJob);
      });
    }
  }

  void onError(Job job, Error e) {
    developer.log("error: $e");
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
