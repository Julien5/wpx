import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/future_rendering_widget.dart';

class RendererParameters {
  final Set<InputType> kinds;
  final TrackData trackData;
  const RendererParameters({required this.kinds, required this.trackData});
  ValueKey createKey() {
    final sortedKinds = kinds.map((k) => k.toString()).toList()..sort();
    return ValueKey('${trackData.toString()}|${sortedKinds.join(",")}');
  }
}

class SvgWidget extends StatefulWidget {
  final RendererParameters parameters;
  final Size rendererSize;
  const SvgWidget({
    super.key,
    required this.rendererSize,
    required this.parameters,
  });

  @override
  State<SvgWidget> createState() => _SvgWidgetState();
}

class _SvgWidgetState extends State<SvgWidget> {
  FutureRenderer? renderer;
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => _initState());
  }

  void _initState() {
    assert(mounted);
    SegmentModel model = Provider.of<SegmentModel>(context, listen: false);
    renderer = model.giveRenderer(
      widget.parameters.kinds,
      widget.parameters.trackData,
    );
    if (widget.parameters.trackData == TrackData.wheel) {
      renderer!.setSize(widget.rendererSize);
    } else {
      renderer!.setSize(widget.rendererSize * 1.5);
    }
    renderer!.addListener(_onRendererChanged);
    _onRendererChanged();
  }

  @override
  void dispose() {
    super.dispose();
    renderer!.removeListener(_onRendererChanged);
  }

  void _onRendererChanged() {
    developer.log("[_onRendererChanged] _onRendererChanged");
    if (!mounted) {
      return;
    }
    assert(renderer != null);
    setState(() {
      if (renderer!.needsStart()) {
        renderer!.start();
      }
    });
  }

  @override
  Widget build(BuildContext ctx) {
    if (renderer == null) {
      return Text("hi");
    }
    return FutureRenderingWidget(future: renderer!, interactive: false);
  }
}

class LayoutWidget extends StatelessWidget {
  final RendererParameters parameters;
  const LayoutWidget({super.key, required this.parameters});

  static LayoutWidget make(Set<InputType> kinds, TrackData trackData) {
    RendererParameters p = RendererParameters(
      kinds: kinds,
      trackData: trackData,
    );
    return LayoutWidget(key: p.createKey(), parameters: p);
  }

  @override
  Widget build(BuildContext ctx) {
    return LayoutBuilder(
      builder: (BuildContext context, BoxConstraints constraints) {
        Size size = constraints.biggest;
        developer.log("ProfileWidget size: $size");
        return SvgWidget(rendererSize: size, parameters: parameters);
      },
    );
  }
}

class WhiteWidget extends StatelessWidget {
  final Color color;
  const WhiteWidget({super.key, required this.color});

  @override
  Widget build(BuildContext ctx) {
    return LayoutBuilder(
      builder: (BuildContext context, BoxConstraints constraints) {
        Size size = constraints.biggest;
        developer.log("WhiteWidget size: $size");
        final double width =
            size.width.isFinite ? size.width : constraints.maxWidth;
        final double height =
            size.height.isFinite ? size.height : constraints.maxHeight;
        return Container(width: width, height: height, color: color);
      },
    );
  }
}

class SideIconButton extends StatelessWidget {
  final VoidCallback? onPressed;
  final TrackData trackData;
  const SideIconButton({super.key, required this.trackData, this.onPressed});

  final double size = 30;
  final double margin = 8;
  Image icon(TrackData data) {
    String filename = 'icons/png/map.png';
    if (data == TrackData.wheel) {
      filename = 'icons/png/clock.png';
    }
    if (data == TrackData.profile) {
      filename = 'icons/png/profile.png';
    }
    return Image.asset(filename, width: size - margin, height: size - margin);
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      width: size,
      height: size,
      decoration: BoxDecoration(
        color: Colors.white,
        border: Border.all(color: Colors.black, width: 3.0),
        borderRadius: BorderRadius.circular(margin),
      ),
      child: IconButton(
        padding: EdgeInsets.zero,
        constraints: BoxConstraints.tight(Size(size, size)),
        icon: icon(trackData),
        onPressed: onPressed,
      ),
    );
  }
}

class TrackMultiView extends StatefulWidget {
  final Set<InputType> kinds;
  const TrackMultiView({super.key, required this.kinds});

  @override
  State<TrackMultiView> createState() => _TrackMultiViewState();
}

class _TrackMultiViewState extends State<TrackMultiView> {
  List<Widget> widgets = [Text("loading"), Text("loading"), Text("loading")];

  @override
  void initState() {
    super.initState();
    widgets.clear();
    widgets.add(LayoutWidget.make(widget.kinds, TrackData.profile));
    widgets.add(LayoutWidget.make(widget.kinds, TrackData.map));
    widgets.add(WhiteWidget(key: const ValueKey('white'), color: Colors.white));
    widgets.add(LayoutWidget.make(widget.kinds, TrackData.wheel));
  }

  void onTap() {
    return;
    setState(() {
      cycleToFront();
    });
  }

  void cycleToFront() {
    int end = 3;
    Widget current = widgets[end];
    widgets[end] = widgets[1];
    widgets[1] = widgets[0];
    widgets[0] = current;
  }

  void bringToFront(int index) {
    int end = 3;
    Widget current = widgets[end];
    widgets[end] = widgets[index];
    widgets[index] = current;
  }

  void onButtonPressed(TrackData data) {
    int index = widgets.indexWhere((widget) {
      return widget is LayoutWidget && widget.parameters.trackData == data;
    });
    developer.log("index = $index");
    setState(() {
      bringToFront(index);
    });
  }

  @override
  Widget build(BuildContext ctx) {
    double margin = 8;
    IconButton(
      icon: Image.asset('assets/images/profile.png', width: 20, height: 20),
      onPressed: () {},
    );
    Widget buttons = Positioned.fill(
      right: 8,
      child: Column(
        mainAxisAlignment: MainAxisAlignment.spaceEvenly,
        crossAxisAlignment: CrossAxisAlignment.end,
        children: [
          SideIconButton(
            trackData: TrackData.map,
            onPressed: () => onButtonPressed(TrackData.map),
          ),
          SideIconButton(
            trackData: TrackData.profile,
            onPressed: () => onButtonPressed(TrackData.profile),
          ),
          SideIconButton(
            trackData: TrackData.wheel,
            onPressed: () => onButtonPressed(TrackData.wheel),
          ),
        ],
      ),
    );

    return GestureDetector(
      onTap: onTap,
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxHeight: 200),
        child: Padding(
          padding: EdgeInsetsGeometry.fromLTRB(margin, margin, margin, margin),
          child: SizedBox(
            width: double.infinity,
            height: 200,
            child: Stack(children: [...widgets, buttons]),
          ),
        ),
      ),
    );
  }
}
