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

RenderingsProvider createRenderingsProviders(Renderers r, Widget child) {
  return RenderingsProvider(r, child);
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
      var w = createRenderingsProviders(renderer, SegmentStack());
      w.renderers.trackRendering.start();
      _segments.add(w);
    }
  }

  @override
  Widget build(BuildContext context) {
    developer.log("[segments] [build] #segments=${_segments.length}");
    return ListView.separated(
      itemCount: _segments.length,
      separatorBuilder: (context, index) => const Divider(),
      itemBuilder: (context, index) {
        return _segments[index];
      },
    );
  }
}

class SegmentsConsumer extends StatelessWidget {
  const SegmentsConsumer({super.key});

  @override
  Widget build(BuildContext ctx) {
    return Consumer<SegmentsProvider>(
      builder: (context, segmentsProvider, child) {
        SegmentsProvider provider = Provider.of<SegmentsProvider>(
          context,
          listen: false,
        );
        return Center(
          child: Container(
            constraints: const BoxConstraints(maxWidth: 1400), // Set max width to 1400
            child: Column(
              children: [
                Buttons(more: provider.decrementDelta, less: provider.incrementDelta),
                Expanded(child: SegmentsView(segmentsProvider: provider)),
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