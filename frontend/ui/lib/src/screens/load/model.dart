import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;
import 'package:ui/src/screens/home/home_screen.dart';

enum Job { gpx, osm, controls, none }

class FutureJob {
  final Future<void> future;
  final Job job;

  FutureJob({required this.future, required this.job});
}

class LoadScreenModel extends ChangeNotifier {
  Set<Job> done = {};
  final Map<Job, bridge.Error> _failed = {};
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
      future = root.getBridge().loadControls(
        source: bridge.ControlSource.waypoints,
      );
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
    makeFuture(job);
    developer.log("future created");
    notifyListeners();
  }

  bridge.SegmentStatistics? _statistics;
  void onCompleted(Job job) {
    if (job == Job.gpx) {
      _statistics = root.statistics();
    } else if (job == Job.controls) {
      _controls = root.getBridge().getWaypoints(
        segment: root.trackSegment(),
        kinds: {bridge.InputType.control},
      );
    }
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

  bridge.SegmentStatistics statistics() {
    return _statistics!;
  }

  bool doneAll() {
    return done.contains(Job.gpx) &&
        done.contains(Job.controls) &&
        done.contains(Job.osm);
  }

  void onError(Job job, bridge.Error e) {
    developer.log("error: $e");
    _failed[job] = e;
    notifyListeners();
  }

  List<bridge.Waypoint>? _controls;
  int controlsCount() {
    assert(done.contains(Job.controls));
    return _controls!.length;
  }

  String _lastEvent = "";

  void onChanged(RootModel root, EventModel event) {
    developer.log("LoadScreenModel::onRootChanged");
    _lastEvent = event.get();
    notifyListeners();
  }

  String lastEvent() {
    return _lastEvent;
  }

  bridge.Error? error(Job job) {
    if (!_failed.containsKey(job)) {
      return null;
    }
    return _failed[job]!;
  }
}
