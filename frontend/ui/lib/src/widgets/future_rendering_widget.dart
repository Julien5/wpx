import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/log.dart';
//import 'package:flutter_svg/flutter_svg.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/svgelements.dart';
import 'package:ui/src/widgets/interactive_svg_widget.dart';
import 'package:ui/src/widgets/static_svg_widget.dart';
import 'package:visibility_detector/visibility_detector.dart';

class FutureRenderingWidget extends StatefulWidget {
  final bool interactive;
  const FutureRenderingWidget({super.key, required this.interactive});

  @override
  State<FutureRenderingWidget> createState() => _FutureRenderingWidgetState();
}

class _FutureRenderingWidgetState extends State<FutureRenderingWidget> {
  Widget? svgWidget;
  VisibilityInfo? visibilityInfo;

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

  @override
  Widget build(BuildContext context) {
    FutureRenderer future = Provider.of<FutureRenderer>(context);
    return buildWorker(future);
  }
}
