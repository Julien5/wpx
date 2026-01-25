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
  Map<Job, bridge.Error> errors = {};
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

  void onError(Job job, bridge.Error e) {
    developer.log("error: $e");
    errors[job] = e;
    notifyListeners();
  }

  bridge.SegmentStatistics? statistics() {
    if (!root.getBridge().isLoaded()) {
      developer.log("bridge not loaded");
      return null;
    }
    developer.log("bridge loaded");
    return root.statistics();
  }

  int controlsCount() {
    List<bridge.Waypoint> w = root.getBridge().getWaypoints(
      segment: root.trackSegment(),
      kinds: {bridge.InputType.control},
    );
    return w.length;
  }

  void onRootChanged(RootModel root) {
    developer.log("LoadScreenModel::onRootChanged");
    notifyListeners();
  }
}
