import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/futurerenderer.dart';
import 'package:wpx/src/models/kindsmodel.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/models/screen_configuration.dart';
import 'package:wpx/src/models/segmentmodel.dart';

class MidColumn extends StatelessWidget {
  final List<Widget> children;

  const MidColumn({super.key, required this.children});

  @override
  Widget build(BuildContext ctx) {
    return Column(mainAxisSize: MainAxisSize.min, children: children);
  }
}

class MobileScaffoldBody extends StatefulWidget {
  final List<BridgeRenderFunction> clients;
  final Widget topRow;
  final Widget midColumn;
  final ScreenFocus screenFocus;
  const MobileScaffoldBody({
    super.key,
    required this.topRow,
    required this.midColumn,
    required this.screenFocus,
    required this.clients,
  });

  @override
  State<MobileScaffoldBody> createState() => _MobileScaffoldBodyState();
}

class _MobileScaffoldBodyState extends State<MobileScaffoldBody> {
  FutureRenderer? futureRenderer;

  bool isVisible(FociModel fociModel) {
    if (widget.screenFocus == ScreenFocus.overview) {
      return fociModel.hasOnly(ScreenFocus.overview);
    }
    return fociModel.contains(widget.screenFocus);
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    FociModel fociModel = context.watch<FociModel>();
    context.watch<ParameterModel>();
    KindsModel kindsModel = Provider.of(context);
    if (futureRenderer == null) {
      SegmentModel segmentModel = Provider.of(context);
      debugPrint("CREATE FUTURE RENDER FOR ${widget.screenFocus}");
      futureRenderer = FutureRenderer(
        bridge: segmentModel.backend,
        segment: segmentModel.segment,
        clients: widget.clients,
        kinds: kindsModel.kinds,
        name: "${widget.screenFocus}",
      );
    } else {
      debugPrint("REUSE FUTURE RENDER FOR ${widget.screenFocus}");
    }
    futureRenderer!.setKinds(kindsModel.kinds);
    futureRenderer!.setVisible(isVisible(fociModel));
    // Needed because futureRenderer does not know the time parameters.
    // Change time parameters => update graphics.
    futureRenderer!.reset();
  }

  @override
  Widget build(BuildContext ctx) {
    ScreenConfiguration screen = Provider.of<ScreenConfiguration>(ctx);
    Widget child = HorizontalLayout(
      topRow: widget.topRow,
      midColumn: widget.midColumn,
    );
    if (screen.mode == DisplayMode.vertical) {
      child = VerticalLayout(
        topRow: widget.topRow,
        midColumn: widget.midColumn,
      );
    }
    return FutureRendererProvider(
      futureRenderer: futureRenderer!,
      child: child,
    );
  }
}

class HorizontalLayout extends StatelessWidget {
  final Widget topRow;
  final Widget midColumn;
  const HorizontalLayout({
    super.key,
    required this.topRow,
    required this.midColumn,
  });

  @override
  Widget build(BuildContext ctx) {
    return LayoutBuilder(
      builder: (context, constraints) {
        const double height = 400;
        const double midSpace = 30;
        final double availWidth = 800 + midSpace;

        // we should take the constraints into account.

        List<Widget> children = [
          ConstrainedBox(
            constraints: BoxConstraints(maxHeight: 400, maxWidth: 400),
            child: topRow,
          ),
          SizedBox(width: midSpace),
          ConstrainedBox(
            constraints: BoxConstraints(maxHeight: 380, maxWidth: 400),
            child: midColumn,
          ),
        ];

        return Align(
          alignment: Alignment.topCenter,
          child: Padding(
            padding: EdgeInsets.fromLTRB(10, 0, 10, 0),
            child: ConstrainedBox(
              constraints: BoxConstraints(maxHeight: height),
              child: SingleChildScrollView(
                scrollDirection: Axis.vertical,
                child: ConstrainedBox(
                  constraints: BoxConstraints(maxWidth: availWidth),
                  child: Row(
                    crossAxisAlignment: CrossAxisAlignment.center,
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: children,
                  ),
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}

class VerticalLayout extends StatelessWidget {
  final Widget topRow;
  final Widget midColumn;
  const VerticalLayout({
    super.key,
    required this.topRow,
    required this.midColumn,
  });

  @override
  Widget build(BuildContext ctx) {
    const double colWidth = 400;

    List<Widget> children = [
      ConstrainedBox(
        constraints: BoxConstraints(maxHeight: 200),
        child: topRow,
      ),
      ConstrainedBox(
        constraints: BoxConstraints(maxHeight: 380),
        child: midColumn,
      ),
    ];

    return Align(
      alignment: Alignment.topCenter,
      child: Padding(
        padding: EdgeInsets.fromLTRB(10, 0, 10, 0),
        child: ConstrainedBox(
          constraints: BoxConstraints(maxWidth: colWidth),
          child: SingleChildScrollView(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.start,
              children: children,
            ),
          ),
        ),
      ),
    );
  }
}
