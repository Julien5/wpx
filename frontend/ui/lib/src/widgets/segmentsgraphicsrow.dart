import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/futurerenderer.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/models/stackviewscontroller.dart';
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bride;
import 'package:wpx/src/utils/utils.dart';

import 'segmentgraphics.dart';

class LocalSegmentGraphics extends StatefulWidget {
  final Kinds kinds;
  final SegmentModel model;

  const LocalSegmentGraphics({
    super.key,
    required this.kinds,
    required this.model,
  });

  @override
  State<LocalSegmentGraphics> createState() => _LocalSegmentGraphicsState();
}

class _LocalSegmentGraphicsState extends State<LocalSegmentGraphics> {
  FutureRenderer? futureRenderer;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    futureRenderer ??= FutureRenderer(
      bridge: widget.model.backend,
      segment: widget.model.segment,
      kinds: widget.kinds,
      clients: [TrackData.map, TrackData.profile],
      name: "LocalSegmentGraphics",
    );
    futureRenderer!.setVisible(true);
    futureRenderer!.updateSegment(widget.model.segment);
  }

  @override
  void dispose() {
    super.dispose();
    futureRenderer!.dispose();
  }

  @override
  Widget build(BuildContext context) {
    context.watch<ParameterModel>();
    developer.log("[LocalSegmentGraphics]");
    widget.model.debug();
    debugPrint("update segment: ${widget.model.segment.id()}");
    assert(futureRenderer != null);
    return MultiProvider(
      providers: [
        ChangeNotifierProvider.value(value: widget.model),
        ChangeNotifierProvider.value(value: futureRenderer),
      ],
      child: SegmentGraphics(kinds: widget.kinds),
    );
  }
}

class SegmentSelector extends StatefulWidget {
  final TabController tabController;
  final List<SegmentModel> segments;
  final Kinds kinds;
  const SegmentSelector({
    super.key,
    required this.tabController,
    required this.segments,
    required this.kinds,
  });

  @override
  State<SegmentSelector> createState() => _SegmentSelectorState();
}

class _SegmentSelectorState extends State<SegmentSelector> {
  @override
  Widget build(BuildContext context) {
    List<Widget> children = [];
    for (SegmentModel model in widget.segments) {
      children.add(
        Center(child: LocalSegmentGraphics(model: model, kinds: widget.kinds)),
      );
    }
    return Column(
      children: [
        Expanded(
          child: TabBarView(
            controller: widget.tabController,
            children: children,
          ),
        ),
      ],
    );
  }
}

class SegmentsGraphicsRow extends StatefulWidget {
  final Kinds kinds;
  final double height;
  const SegmentsGraphicsRow({
    super.key,
    required this.kinds,
    required this.height,
  });

  @override
  State<SegmentsGraphicsRow> createState() => _SegmentsGraphicsRowState();
}

class _SegmentsGraphicsRowState extends State<SegmentsGraphicsRow>
    with TickerProviderStateMixin {
  TabController? _tabController;
  List<SegmentModel> segments = [];
  ParameterModel? parameterModel;

  void _onParameterChanged() {
    bride.Bridge backend = getBackend(context);
    List<Segment> newSegments = backend.segments();
    int oldLength = segments.length;
    int newLength = newSegments.length;
    developer.log("_onRootChanged: new length:$newLength");
    if (oldLength != newLength) {
      segments.clear();
    } else {
      return;
    }
    for (Segment segment in newSegments) {
      SegmentModel model = SegmentModel(backend: backend, segment: segment);
      segments.add(model);
    }
    _tabController = TabController(length: segments.length, vsync: this);
    setState(() {});
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (parameterModel == null) {
      parameterModel = Provider.of<ParameterModel>(context, listen: false);
      parameterModel!.addListener(_onParameterChanged);
    }
    _onParameterChanged();
    assert(_tabController != null);
  }

  @override
  void dispose() {
    if (_tabController != null) {
      _tabController!.dispose();
    }
    if (parameterModel != null) {
      parameterModel!.removeListener(_onParameterChanged);
    }
    super.dispose();
  }

  void onButtonPressed(BuildContext context, TrackData data) {
    StackViewsController model = Provider.of<StackViewsController>(
      context,
      listen: false,
    );
    model.changeCurrent(data);
  }

  @override
  Widget build(BuildContext context) {
    developer.log("[rebuild _SegmentsGraphicsRowState]");
    StackViewsController model = Provider.of<StackViewsController>(context);
    assert(_tabController != null);

    TrackData currentModelData = model.currentData();

    Widget buttonColumn = SegmentGraphicsButtonsColumn(
      onButtonPressed: (trackData) => {onButtonPressed(context, trackData)},
      selected: currentModelData,
      size: 30,
    );

    Widget graphics = Padding(
      padding: EdgeInsetsGeometry.fromLTRB(0, 0, 5, 0),
      child: ConstrainedBox(
        constraints: BoxConstraints(maxHeight: widget.height),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Expanded(
              child: SegmentSelector(
                tabController: _tabController!,
                segments: segments,
                kinds: widget.kinds,
              ),
            ),
            //Expanded(child: Center(child: SegmentGraphics(kinds: allkinds()))),
            buttonColumn,
          ],
        ),
      ),
    );

    return Column(
      children: [graphics, TabPageSelector(controller: _tabController)],
    );
  }
}
