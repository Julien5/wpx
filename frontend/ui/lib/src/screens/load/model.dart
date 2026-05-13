import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bridge;

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

  final bridge.Bridge backend;
  final RootModel rootModel;
  final EventModel events;
  final UserInput userInput;
  bool retryOsm = false;
  int _osmRetryCount = 0;
  List<bridge.TrackPart>? _trackParts;

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
    // The UI should not allow leaving LoadScreen while a job is running.
    if (runningJob() != Job.none && runningJob() == Job.osm) {
      cancelOsm();  
    }
    super.dispose();
  }

  bool needsStart() {
    return runningFuture == null && done.isEmpty && _failed.isEmpty;
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
      trackPartsFuture = Future<List<bridge.TrackPart>>(
        () => backend.loadTrackParts(contents: userInput.contents()),
      );
    } else if (job == Job.gpx) {
      assert(_trackParts != null);
      future = Future<void>(() => backend.loadOrdered(parts: _trackParts!));
      //future = backend.loadContents(contents: userInput.contents());
    } else if (job == Job.osm) {
      future = backend.loadOsm();
    } else if (job == Job.controls) {
      future = Future<void>(() => backend.loadControls());
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
    if (runningFuture == null) {
      return Job.none;
    }
    return runningFuture!.job;
  }

  void makeFuture(Job job) {
    SchedulerBinding.instance.addPostFrameCallback((_) {
      _makeFuture(job);
    });
  }

  void start() {
    if (_isDisposed) {
      return;
    }
    retryOsm = true;
    _osmRetryCount = 0;
    done.clear();
    _failed.clear();
    startJob(Job.parts);
  }

  void cancelOsm() {
    if (_isDisposed) {
      return;
    }
    if (runningJob() != Job.osm) {
      debugPrint("running job is not osm");
      return;
    }
    retryOsm = false;
    backend.cancelOsm();
    runningFuture = null;
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
    debugPrint("future created for $job");
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
    runningFuture = null;
    done.add(job);
    _failed.remove(job);
    debugPrint("running notify");
    notifyListeners();

    Job nextJob = next(job);
    if (nextJob != Job.none) {
      Future.delayed(const Duration(milliseconds: 250), () {
        startJob(nextJob);
      });
    }

    if (rootModel.isLoaded()) {
      // TODO: the rootModel should notify autonomously
      debugPrint("rootModel notify");
      rootModel.notify();
      // go to overview.
    }
  }

  bridge.SegmentStatistics statistics() {
    return backend.statistics();
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
    debugPrint("onError: $job isDisposed:$_isDisposed");
    runningFuture = null;
    if (_isDisposed) {
      return;
    }

    if (e is bridge.TrackError) {
      debugPrint("e:${e.toString()}");
      bridge.TrackError trackError = e;
      if (trackError is bridge.TrackError_OSMDownloadTimeout) {
        developer.log("timeout => retry");
        if (retryOsm) {
          Future.delayed(const Duration(seconds: 1), () {
            _failed.remove(job);
            startJob(Job.osm);
            _osmRetryCount += 1;
          });
        }
      }
      if (trackError is bridge.TrackError_OSMDownloadFailed) {
        debugPrint("timeout => should not retry");
        if (retryOsm) {
          Future.delayed(const Duration(seconds: 1), () {
            _failed.remove(job);
            startJob(Job.osm);
            _osmRetryCount += 1;
          });
        }
      }
    }

    developer.log("error: $e");
    _failed[job] = e;
    debugPrint("load notify");
    notifyListeners();
  }

  int osmRetryCount() {
    return _osmRetryCount;
  }

  int controlsCount() {
    assert(done.contains(Job.controls));
    return statistics().controls.length;
  }

  int waypointsCount() {
    assert(done.contains(Job.controls));
    return statistics().waypoints.length;
  }

  String _lastEvent = "";

  void onChanged(RootModel root, EventModel event) {
    debugPrint("LoadScreenModel::onRootChanged");
    _lastEvent = event.get();
    notifyListeners();
  }

  String lastEvent() {
    return _lastEvent;
  }

  Object? hasFailed(Job job) {
    if (!_failed.containsKey(job)) {
      return null;
    }
    return _failed[job]!;
  }

  List<Object> failed() {
    if (_failed.isEmpty) {
      return [];
    }
    return List.from(_failed.keys);
  }
}
