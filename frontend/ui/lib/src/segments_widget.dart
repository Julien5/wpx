import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/counter.dart';
import 'package:ui/src/segment_stack.dart';

class SegmentsView extends StatefulWidget {
  final SegmentsProvider? segmentsProvider;
  const SegmentsView({super.key, this.segmentsProvider});

  @override
  State<SegmentsView> createState() => SegmentsViewState();
}

class RenderingsProvider extends MultiProvider {
  final Renderers renderers;

  RenderingsProvider(Renderers r, Widget child, {super.key})
    : renderers = r,
      super(
        providers: [
          ChangeNotifierProvider.value(value: r.trackRendering),
          ChangeNotifierProvider.value(value: r.waypointsRendering),
        ],
        child: child,
      );
}

class SegmentsViewState extends State<SegmentsView> {
  final List<RenderingsProvider> _segments = [];

  @override
  void initState() {
    super.initState();
    _initRenderingProviders(widget.segmentsProvider!);
  }

  void _initRenderingProviders(SegmentsProvider segmentsProvider) {
    var S = segmentsProvider.segments();
    assert(_segments.isEmpty);
    for (var renderer in S) {
      var w = RenderingsProvider(renderer, SegmentStack());
      w.renderers.trackRendering.start();
      _segments.add(w);
    }
  }

  @override
  Widget build(BuildContext context) {
    developer.log("[segments] [build] #segments=${_segments.length}");
    List<Tab> tabs=[];
    for (var s in _segments) {
      var id=s.renderers.trackRendering.segment.id();
      tabs.add(Tab(text: "segment ${id.toInt()}"));
    }
    return MaterialApp(
      home: DefaultTabController(
        length: _segments.length,
        child: Scaffold(
          appBar: AppBar(
            bottom: TabBar(
              tabs:tabs,
            ),
          ),
          body: TabBarView(
            children: _segments,
          ),
        ),
      ),
    );
  }
}

class SegmentsConsumer extends StatelessWidget {
  const SegmentsConsumer({super.key});

  @override
  Widget build(BuildContext ctx) {
    return Consumer<SegmentsProvider>(
      builder: (context, segmentsProvider, child) {
        return Center(
          child: Container(
            constraints: const BoxConstraints(maxWidth: 1500), 
            child: Column(
              children: [
                Buttons(more: segmentsProvider.decrementDelta, less: segmentsProvider.incrementDelta),
                Expanded(child: SegmentsView(segmentsProvider: segmentsProvider)),
              ],
            ),
          ),
        );
      },
    );
  }
}


class Buttons extends StatelessWidget {
  final VoidCallback more;
  final VoidCallback less;
  const Buttons({super.key, required this.more, required this.less});

  @override
  Widget build(BuildContext ctx) {
    return Column(
          children: [
            Row(
              children: [
                PressButton(
                  label: "more",
                  onCounterPressed: more,
                ),
                PressButton(
                  label: "less",
                  onCounterPressed: less,
                ),
              ],
            ),
          ]);
  }
}
