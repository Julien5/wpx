import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/models/trackviewswitch.dart';
import 'package:ui/src/rust/api/bridge.dart';

import 'segmentgraphics.dart';

class LocalSegmentGraphics extends StatelessWidget {
  final Kinds kinds;
  final SegmentModel model;

  const LocalSegmentGraphics({
    super.key,
    required this.kinds,
    required this.model,
  });
  @override
  Widget build(BuildContext context) {
    return ChangeNotifierProvider.value(
      value: model,
      child: SegmentGraphics(kinds: kinds),
    );
  }
}

class SegmentSelector extends StatefulWidget {
  final TabController tabController;
  final List<SegmentModel> segments;
  const SegmentSelector({
    super.key,
    required this.tabController,
    required this.segments,
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
        Center(child: LocalSegmentGraphics(model: model, kinds: allkinds())),
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
  final Set<InputType> kinds;
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
    with SingleTickerProviderStateMixin {
  TabController? _tabController;
  List<SegmentModel> segments = [];

  @override
  void initState() {
    super.initState();
    developer.log("initState");
    SchedulerBinding.instance.addPostFrameCallback((_) {
      _initState();
    });
  }

  void _initState() {
    RootModel root = Provider.of<RootModel>(context, listen: false);
    developer.log(
      "_SegmentsGraphicsRowState:_initState:${root.segments().length}",
    );
    for (Segment segment in root.segments()) {
      segments.add(SegmentModel(root.getBridge(), segment));
    }
    _tabController = TabController(length: root.segments().length, vsync: this);
    setState(() {});
  }

  @override
  void dispose() {
    if (_tabController != null) {
      _tabController!.dispose();
    }
    super.dispose();
  }

  void onButtonPressed(BuildContext context, TrackData data) {
    TrackViewsSwitch model = Provider.of<TrackViewsSwitch>(
      context,
      listen: false,
    );
    model.changeCurrent(data);
  }

  @override
  Widget build(BuildContext context) {
    TrackViewsSwitch model = Provider.of<TrackViewsSwitch>(context);
    if (_tabController == null) {
      return Text("building tab controller");
    }

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
