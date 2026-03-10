import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

enum Job { parts, gpx, osm, controls, none }

class FutureJob {
  final Future<void>? future;
  final Future<List<bridge.TrackPart>>? partsFuture;
  final Job job;

  FutureJob({required this.job, this.future, this.partsFuture});

  static FutureJob normal(Job job, Future<void> future) {
    return FutureJob(job: job, future: future);
  }

  static FutureJob parts(Job job, Future<List<bridge.TrackPart>> future) {
    return FutureJob(job: job, partsFuture: future);
  }
}

class LoadScreenModel extends ChangeNotifier {
  Set<Job> done = {};
  final Map<Job, Object> _failed = {};
  Job? running;

  final bridge.Bridge backend;
  final RootModel rootModel;
  final EventModel events;
  final UserInput userInput;
  List<bridge.Waypoint>? _controls;
  List<bridge.TrackPart>? _trackParts;
  bridge.SegmentStatistics? _statistics;

  FutureJob? runningFuture;
  LoadScreenModel({
    required this.backend,
    required this.rootModel,
    required this.events,
    required this.userInput,
  }) {
    debugPrint('LoadScreenModel created');
  }
  bool _isDisposed = false;
  @override
  void dispose() {
    _isDisposed = true;
    // TODO: cancel osm task if it is running, otherwise it
    // blocks as soon as a &mut self function is called.
    super.dispose();
  }

  bool needsStart() {
    return running == null && done.isEmpty;
  }

  bool hasDone(Job job) {
    return done.contains(job);
  }

  static Job next(Job old) {
    if (old == Job.parts) {
      return Job.gpx;
    }
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
    Future<List<bridge.TrackPart>>? trackPartsFuture;
    if (job == Job.parts) {
      trackPartsFuture = backend.loadTrackParts(contents: userInput.contents());
    } else if (job == Job.gpx) {
      assert(_trackParts != null);
      future = backend.loadOrdered(parts: _trackParts!);
      //future = backend.loadContents(contents: userInput.contents());
    } else if (job == Job.osm) {
      future = backend.loadOsm();
    } else if (job == Job.controls) {
      future = backend.loadControls(source: bridge.ControlSource.segments);
    } else {
      assert(false);
    }
    if (future != null) {
      future.then((_) => onCompleted(job)).catchError((error) {
        onError(job, error);
      });
      runningFuture = FutureJob.normal(job, future);
    } else if (trackPartsFuture != null) {
      trackPartsFuture.then((list) => onPartsCompleted(job, list)).catchError((
        error,
      ) {
        onError(job, error);
      });
      runningFuture = FutureJob.parts(job, trackPartsFuture);
    }
  }

  Job runningJob() {
    if (running == null) {
      return Job.none;
    }
    return running!;
    //return runningFuture!.job;
  }

  void makeFuture(Job job) {
    running = job;
    SchedulerBinding.instance.addPostFrameCallback((_) {
      _makeFuture(job);
    });
  }

  void start() {
    if (_isDisposed) {
      return;
    }
    startJob(Job.parts);
  }

  void retry(Job job) {
    if (_isDisposed) {
      return;
    }
    done.remove(job);
    _failed.remove(job);
    startJob(job);
  }

  void startJob(Job job) {
    if (_isDisposed) {
      return;
    }
    done.remove(job);
    if (next(job) != Job.none) {
      done.remove(next(job));
    }

    makeFuture(job);
    developer.log("future created");
    notifyListeners();
  }

  void onPartsCompleted(Job job, List<bridge.TrackPart> parts) {
    if (_isDisposed) {
      return;
    }
    assert(job == Job.parts);
    _trackParts = parts;
    onCompleted(job);
  }

  void onCompleted(Job job) {
    if (_isDisposed) {
      return;
    }
    if (job == Job.gpx) {
      _statistics = backend.statistics();
    } else if (job == Job.controls) {
      _controls = backend.getWaypoints(
        segment: backend.trackSegment(),
        kinds: {bridge.Kind.controls},
      );
    }

    running = null;
    runningFuture = null;
    done.add(job);
    debugPrint("running notify");
    notifyListeners();

    Job nextJob = next(job);
    if (nextJob != Job.none) {
      Future.delayed(const Duration(milliseconds: 250), () {
        startJob(nextJob);
      });
    } else if (doneAll()) {
      rootModel.notify();
      // go to overview.
    }
  }

  bridge.SegmentStatistics statistics() {
    return _statistics!;
  }

  List<bridge.TrackPart> parts() {
    return _trackParts!;
  }

  void reorderParts(int oldIndex, int newIndex) {
    assert(_trackParts != null);
    debugPrint("reorder: $oldIndex => $newIndex");
    for (bridge.TrackPart part in _trackParts!) {
      debugPrint("before part: ${part.partIndex} ${part.name}");
    }
    final bridge.TrackPart element = _trackParts!.removeAt(oldIndex);
    _trackParts!.insert(newIndex, element);
    for (bridge.TrackPart part in _trackParts!) {
      debugPrint("after part: ${part.partIndex} ${part.name}");
    }
    startJob(Job.gpx);
  }

  bool doneAll() {
    return done.contains(Job.gpx) &&
        done.contains(Job.controls) &&
        done.contains(Job.osm);
  }

  void onError(Job job, Object e) {
    if (_isDisposed) {
      return;
    }

    if (error is bridge.TrackError) {
      // Now you can handle your specific Rust variants
      developer.log("onError:${error.toString()}");
    }

    developer.log("error: $e");
    _failed[job] = e;
    notifyListeners();
  }

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

  Object? error(Job job) {
    if (!_failed.containsKey(job)) {
      return null;
    }
    return _failed[job]!;
  }
}
