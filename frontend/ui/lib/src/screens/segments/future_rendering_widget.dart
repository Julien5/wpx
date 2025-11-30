import 'package:flutter/material.dart';
import 'package:ui/src/log.dart';
//import 'package:flutter_svg/flutter_svg.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/svgelements.dart';
import 'package:ui/src/widgets/interactive_svg_widget.dart';
import 'package:ui/src/widgets/static_svg_widget.dart';

class FutureRenderingWidget extends StatefulWidget {
  final FutureRenderer future;
  final bool interactive;
  const FutureRenderingWidget({
    super.key,
    required this.future,
    required this.interactive,
  });

  @override
  State<FutureRenderingWidget> createState() => _FutureRenderingWidgetState();
}

class _FutureRenderingWidgetState extends State<FutureRenderingWidget> {
  Widget? svgWidget;

  Widget buildWorker() {
    if (widget.future.done()) {
      log("[render-parse-start:${widget.future.trackData}]");
      SvgRootElement svgRootElement = parseSvg(widget.future.result());
      log("[render-parse-end:${widget.future.trackData}]");

      if (!widget.interactive) {
        svgWidget = StaticSvgWidget(svgRootElement: svgRootElement);
      } else {
        svgWidget = SvgWidget(svgRootElement: svgRootElement);
      }
    }
    if (!widget.future.done() && svgWidget == null) {
      return Text("starting ${widget.future.trackData} ${widget.future.id()}");
    }

    if (!widget.future.done()) {
      return Stack(
        children: <Widget>[
          Text("updating ${widget.future.trackData} ${widget.future.id()}"),
          svgWidget!,
        ],
      );
    }
    return svgWidget!;
  }

  @override
  Widget build(BuildContext context) {
    return buildWorker();
  }
}
