import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/trackviewswitch.dart';
import 'package:ui/src/rust/api/bridge.dart';

import 'segmentgraphics.dart';

class SegmentSelector extends StatefulWidget {
  final TabController tabController;
  const SegmentSelector({super.key, required this.tabController});

  @override
  State<SegmentSelector> createState() => _SegmentSelectorState();
}

class _SegmentSelectorState extends State<SegmentSelector> {
  @override
  void initState() {
    super.initState();
    developer.log("SegmentSelector init staete");
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Expanded(
          child: TabBarView(
            controller: widget.tabController,
            children: [
              Center(child: SegmentGraphics(kinds: allkinds())),
              Center(child: SegmentGraphics(kinds: {InputType.userStep})),
              Center(child: SegmentGraphics(kinds: {InputType.osm})),
            ],
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
  late TabController _tabController;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 3, vsync: this);
  }

  @override
  void dispose() {
    _tabController.dispose();
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
    developer.log("rebuild view");

    TrackData currentModelData = model.currentData();

    Widget buttonColumn = SegmentGraphicsButtonsColumn(
      onButtonPressed: (trackData) => {onButtonPressed(context, trackData)},
      selected: currentModelData,
      size: 30,
    );

    Widget graphics = Padding(
      padding: EdgeInsetsGeometry.fromLTRB(0, 0, 5, 0),
      child: ConstrainedBox(
        constraints: BoxConstraints(maxHeight: 200),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Expanded(child: SegmentSelector(tabController: _tabController)),
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
