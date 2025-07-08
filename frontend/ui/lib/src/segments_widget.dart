import 'dart:developer' as developer;

import 'package:file_picker/file_picker.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/counter.dart';
import 'package:ui/src/segment_stack.dart';

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

class SegmentsView extends StatefulWidget {
  final SegmentsProvider? segmentsProvider;
  const SegmentsView({super.key, this.segmentsProvider});

  @override
  State<SegmentsView> createState() => SegmentsViewState();
}

class SegmentsViewState extends State<SegmentsView> {
  final List<RenderingsProvider> _segments = [];

  @override
  void initState() {
    super.initState();
    _initRenderingProviders(widget.segmentsProvider!);
  }

  void _initRenderingProviders(SegmentsProvider segmentsProvider) {
    developer.log("[_initRenderingProviders]");
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
    List<Tab> tabs = [];
    for (var s in _segments) {
      var id = s.renderers.trackRendering.segment.id();
      tabs.add(Tab(text: "segment ${id.toInt()}"));
    }
    return DefaultTabController(
      length: _segments.length,
      child: Scaffold(
        appBar: TabBar(tabs: tabs),
        body: TabBarView(children: _segments),
      ),
    );
  }
}

class FindGPXFile extends StatelessWidget {
  final SegmentsProvider segmentsProvider;
  const FindGPXFile({super.key, required this.segmentsProvider});

  void chooseGPX() async {
    FilePickerResult? result = await FilePicker.platform.pickFiles(
      type: FileType.any,
    );
    if (result != null) {
      developer.log("result: ${result.count}");
      for (var file in result.files) {
        if (!kIsWeb) {
          segmentsProvider.setFilename(file.path!);
        } else {
          segmentsProvider.setContent(file.bytes!.buffer.asInt8List().toList());
        }
        return;
      }
    }
  }

  void chooseDemo() async {
    segmentsProvider.setDemoContent();
  }

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          ElevatedButton(
            onPressed: chooseGPX,
            child: const Text("Choose GPX file"),
          ),
          const SizedBox(height: 20),
          ElevatedButton(onPressed: chooseDemo, child: const Text("Demo")),
        ],
      ),
    );
  }
}

class SegmentsConsumer extends StatelessWidget {
  const SegmentsConsumer({super.key});

  Widget childrenChoose(BuildContext ctx, SegmentsProvider provider) {
    return FindGPXFile(segmentsProvider: provider);
  }

  Widget childrenShow(BuildContext ctx, SegmentsProvider provider) {
    return Column(
      children: [
        Expanded(child: SegmentsView(segmentsProvider: provider)),
      ],
    );
  }

  Widget children(BuildContext ctx, SegmentsProvider provider) {
    if (provider.bridgeIsLoaded()) {
      return childrenShow(ctx, provider);
    }
    return childrenChoose(ctx, provider);
  }

  @override
  Widget build(BuildContext ctx) {
    return Consumer<SegmentsProvider>(
      builder: (context, segmentsProvider, child) {
        developer.log("length=${segmentsProvider.segments().length}");
        return Center(
          child: Container(
            constraints: const BoxConstraints(maxWidth: 1500),
            child: children(ctx, segmentsProvider),
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
            PressButton(label: "more", onCounterPressed: more),
            PressButton(label: "less", onCounterPressed: less),
          ],
        ),
      ],
    );
  }
}
