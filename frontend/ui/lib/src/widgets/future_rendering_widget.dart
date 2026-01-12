import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/log.dart';
//import 'package:flutter_svg/flutter_svg.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/svgelements.dart';
import 'package:ui/src/widgets/interactive_svg_widget.dart';
import 'package:ui/src/widgets/static_svg_widget.dart';
import 'package:visibility_detector/visibility_detector.dart';

class FutureRenderingInnerWidget extends StatefulWidget {
  final bool interactive;
  final FutureRenderer renderer;
  const FutureRenderingInnerWidget({
    super.key,
    required this.interactive,
    required this.renderer,
  });

  @override
  State<FutureRenderingInnerWidget> createState() =>
      _FutureRenderingInnerWidgetState();
}

class _FutureRenderingInnerWidgetState
    extends State<FutureRenderingInnerWidget> {
  Widget? svgWidget;
  VisibilityInfo? visibilityInfo;

  @override
  void initState() {
    super.initState();
    SchedulerBinding.instance.addPostFrameCallback((_) {
      _initState();
    });
  }

  void _onRendererChanged() {
    setState(() {
      if (widget.renderer.needsStart() && isVisible()) {
        widget.renderer.start();
      }
    });
  }

  void _initState() {
    widget.renderer.addListener(_onRendererChanged);
  }

  @override
  void dispose() {
    widget.renderer.removeListener(_onRendererChanged);
    super.dispose();
  }

  Widget buildWorker(FutureRenderer future) {
    if (future.done()) {
      log("[render-parse-start:${future.trackData}]");
      SvgRootElement svgRootElement = parseSvg(future.result());
      log("[render-parse-end:${future.trackData}]");

      if (!widget.interactive) {
        svgWidget = StaticSvgWidget(svgRootElement: svgRootElement);
      } else {
        svgWidget = SvgWidget(svgRootElement: svgRootElement);
      }
    }

    if (!future.done() && svgWidget == null) {
      return Center(child: Text("rendering..."));
    }

    return svgWidget!;
  }

  void _onVisibilityChanged(VisibilityInfo info) {
    if (!mounted) {
      return;
    }
    visibilityInfo = info;
    final future = Provider.of<FutureRenderer>(context, listen: false);

    if (visibilityInfo!.visibleFraction == 0) {
      return;
    }
    future.setSize(visibilityInfo!.size);
    if (visibilityInfo!.visibleFraction > 0 && future.needsStart()) {
      setState(() {
        future.start();
      });
    }
  }

  bool isVisible() {
    return visibilityInfo != null && visibilityInfo!.visibleFraction > 0;
  }

  @override
  Widget build(BuildContext context) {
    FutureRenderer future = Provider.of<FutureRenderer>(context);
    return VisibilityDetector(
      key: Key('future-renderer-${future.id()}'),
      onVisibilityChanged: _onVisibilityChanged,
      child: buildWorker(future),
    );
  }
}

class FutureRenderingWidget extends StatelessWidget {
  final FutureRenderer future;
  final bool interactive;
  const FutureRenderingWidget({
    super.key,
    required this.future,
    required this.interactive,
  });

  @override
  Widget build(BuildContext context) {
    developer.log("(FutureRenderingWidget build) ${future.trackData}");
    return ChangeNotifierProvider.value(
      value: future,
      builder: (context, child) {
        return FutureRenderingInnerWidget(
          interactive: interactive,
          renderer: future,
        );
      },
    );
  }
}
