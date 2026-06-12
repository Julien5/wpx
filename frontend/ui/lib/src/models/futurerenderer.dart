import 'dart:developer' as developer;

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/widgets.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/log.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bridge;
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/utils/utils.dart';

typedef BridgeRenderFunction = bridge.RenderFunction;

class FutureRenderer with ChangeNotifier {
  bridge.Segment _segment;
  final bridge.Bridge _backend;
  final List<BridgeRenderFunction> clients;

  final List<bridge.Kind> _kinds;

  Future<List<bridge.RenderOutput>>? _future;

  final Map<BridgeRenderFunction, Size?> _sizes = {};
  final Map<BridgeRenderFunction, RenderOutput> _results = {};

  bool _visible = false;
  bool _disposed = false;
  final String name;

  FutureRenderer({
    required bridge.Bridge bridge,
    required bridge.Segment segment,
    required this.clients,
    required Kinds kinds,
    required this.name,
  }) : _kinds = [],
       _segment = segment,
       _backend = bridge {
    developer.log("[CREATE FUTURE RENDERER ($segment) ($clients) ($_kinds)]");
    _kinds.addAll(kinds);
    assert(_backend.isLoaded());
  }

  @override
  void dispose() {
    debugPrint("[renderer dispose ($clients)]");
    _future = null; // Clear the future reference
    if (_disposed) {
      return;
    }
    _disposed = true;
    super.dispose();
  }

  void updateSegment(bridge.Segment segment) {
    _segment = segment;
    reset();
  }

  Segment getSegment() {
    return _segment;
  }

  void setKinds(Kinds newkinds) {
    debugPrint("SET KINDS $name: ${newkinds.length} (old:${_kinds.length})");
    if (setEquals(_kinds.toSet(), newkinds.toSet())) {
      return;
    }
    _kinds
      ..clear()
      ..addAll(newkinds);
    reset();
    assert(!done());
  }

  Size getSize(BridgeRenderFunction d) {
    // this size is passed to the backend for rendering
    developer.log("wanted: $d has: ${_sizes.keys}");
    assert(_sizes.containsKey(d));
    assert(_sizes[d] != null);
    return _sizes[d]!;
  }

  bool isVisible() {
    return _visible;
  }

  void setVisible(bool b) {
    _visible = b;
    if (_visible) {
      start();
    }
  }

  void start() {
    debugPrint("[render-request] ($name) ($clients) ($_kinds)");
    if (_disposed) {
      debugPrint(
        "[render-request] ($name) ($clients) abort: renderer disposed",
      );
      return;
    }
    if (_sizes.length != clients.length) {
      debugPrint(
        "[render-request] ($name) ($clients) abort: size is not set for all track data (${_sizes.length} != ${clients.length})",
      );
      return;
    }
    if (!needsStart()) {
      debugPrint(
        "[render-request] ($name) ($clients) abort: renderer does not need start",
      );
      debugPrint(
        "[render-request] ($name) ($clients) abort: started=${started()} && done=${done()} && visible=${isVisible()};",
      );
      return;
    }

    log("[render-request-start(($name)):$clients]");
    _results.clear();
    List<bridge.RenderInput> renderInputs = [];
    for (BridgeRenderFunction d in clients) {
      assert(_sizes.containsKey(d), "missing size for $d at $name");
      (int, int) sizeParameter = sizeAsTuple(makeFinite(_sizes[d]!));
      renderInputs.add(
        bridge.RenderInput(kinds: _kinds, function: d, size: sizeParameter),
      );
    }
    _future = _backend.renderSegment(segment: _segment, inputs: renderInputs);
    _future!.then((values) => onCompleted(values));
  }

  String id() {
    final sortedKinds = _kinds.map((k) => k.toString()).toList()..sort();
    return "${clients.toString()}|${sortedKinds.join(",")}|${_segment.id()}";
  }

  bool started() {
    return _future != null;
  }

  bool needsStart() {
    return !started() && !done() && isVisible();
  }

  void onCompleted(List<bridge.RenderOutput> values) {
    if (_disposed) {
      developer.log("[renderer was disposed]");
      assert(_future == null);
      return;
    }

    developer.log("[onCompleted ($name)]");

    for (bridge.RenderOutput output in values) {
      BridgeRenderFunction d = output.renderInput.function;
      Kinds k = output.renderInput.kinds;

      debugPrint("($name)found value for $d and $k");
      _results[d] = output;
    }

    _future = null;

    log("[render-request-comleted:${_results.keys}]");
    notifyListeners();
  }

  void reset() {
    _future = null;
    _results.clear();
    _sizes.clear();
    assert(!done());
    notifyListeners();
  }

  void restart() {
    _future = null;
    _results.clear();
    start();
  }

  bool setSize(BridgeRenderFunction d, Size newSize) {
    if (newSize == _sizes[d]) {
      return false;
    }
    if (!clients.contains(d)) {
      assert(
        false,
        "setSize for function $d that is not part of this renderer ($name: $clients)",
      );
    }
    debugPrint("setSize($d,$newSize) old:[${_sizes[d]}]");
    _sizes[d] = newSize;
    _future = null;
    _results.clear();
    start();
    return true;
  }

  bool done() {
    return _results.length == clients.length && _results.isNotEmpty;
  }

  String result(BridgeRenderFunction d) {
    assert(done());
    return _results[d]!.svg;
  }

  RenderOutput? renderOutput(BridgeRenderFunction d) {
    return _results[d];
  }
}

class FutureRendererProvider extends StatelessWidget {
  final FutureRenderer futureRenderer;
  final Widget child;

  const FutureRendererProvider({
    super.key,
    required this.futureRenderer,
    required this.child,
  });

  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [ChangeNotifierProvider.value(value: futureRenderer)],
      child: child,
    );
  }
}
