import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/log.dart';
//import 'package:flutter_svg/flutter_svg.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/svgelements.dart';
import 'package:ui/src/widgets/interactive_svg_widget.dart';
import 'package:ui/src/widgets/static_svg_widget.dart';

class FutureRenderingInnerWidget extends StatefulWidget {
  final bool interactive;
  const FutureRenderingInnerWidget({
    super.key,
    required this.interactive,
  });

  @override
  State<FutureRenderingInnerWidget> createState() => _FutureRenderingInnerWidgetState();
}

class _FutureRenderingInnerWidgetState extends State<FutureRenderingInnerWidget> {
  Widget? svgWidget;

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
      return Text("starting ${future.trackData} ${future.id()}");
    }

    if (!future.done()) {
      return Stack(
        children: <Widget>[
          Text("updating ${future.trackData} ${future.id()}"),
          svgWidget!,
        ],
      );
    }
    return svgWidget!;
  }

  @override
  Widget build(BuildContext context) {
    FutureRenderer future = Provider.of<FutureRenderer>(context);
    return buildWorker(future);
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
    return ChangeNotifierProvider.value(
      value: future,
      builder: (context, child) {
        return FutureRenderingInnerWidget(interactive: interactive,);
      },
    );
  }
}
