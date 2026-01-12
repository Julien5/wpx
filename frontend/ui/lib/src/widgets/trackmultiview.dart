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
        Model model = Provider.of<Model>(ctx, listen: false);
        FutureRenderer renderer = model.renderer(parameters.trackData);
        model.setSize(parameters.trackData, size);
        return FutureRenderingWidget(future: renderer, interactive: false);
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
  final TrackData selected;
  final double size;
  const SideIconButton({
    super.key,
    required this.selected,
    required this.size,
    required this.trackData,
    this.onPressed,
  });
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
    double frameWidth = 1.0;
    if (selected == trackData) {
      frameWidth = 3.0;
    }
    return Container(
      width: size,
      height: size,
      decoration: BoxDecoration(
        color: Colors.white,
        border: Border.all(color: Colors.black, width: frameWidth),
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

class View extends StatefulWidget {
  final Set<InputType> kinds;
  const View({super.key, required this.kinds});

  @override
  State<View> createState() => _TrackMultiViewState();
}

class _TrackMultiViewState extends State<View> {
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
    Model model = Provider.of<Model>(context, listen: false);
    model.cycle();
  }

  TrackData currentVisibleData() {
    Widget current = widgets[3];
    if ((current is LayoutWidget) == false) {
      throw Exception("bad widget");
    }
    LayoutWidget l = current as LayoutWidget;
    return l.parameters.trackData;
  }

  void _bringToFront(int index) {
    const int end = 3;
    Widget current = widgets[end];
    widgets[end] = widgets[index];
    widgets[index] = current;
  }

  void updateModel() {
    Model model = Provider.of<Model>(context, listen: false);
    model.changeCurrent(currentVisibleData());
  }

  void onButtonPressed(TrackData data) {
    Model model = Provider.of<Model>(context, listen: false);
    model.changeCurrent(data);
  }

  TrackData currentTrackData() {
    Model model = Provider.of<Model>(context);
    return model.currentData();
  }

  @override
  Widget build(BuildContext ctx) {
    // Instanciating a Provider.of<Model>(context) (listen=true)
    // is necessary to get rebuild on notifyListeners.
    // Provider.of<Model>(context);
    double margin = 8;
    developer.log("rebuild view");
    TrackData currentData = currentTrackData();
    int index = widgets.indexWhere((widget) {
      return widget is LayoutWidget &&
          widget.parameters.trackData == currentData;
    });
    developer.log("index = $index");
    _bringToFront(index);
    const double buttonSize = 30;
    Widget buttons = ConstrainedBox(
      constraints: const BoxConstraints(maxWidth: buttonSize),
      child: Column(
        mainAxisSize: MainAxisSize.max, // Makes Column fill available space
        mainAxisAlignment: MainAxisAlignment.spaceEvenly,
        children: [
          SideIconButton(
            selected: currentData,
            size: buttonSize,
            trackData: TrackData.wheel,
            onPressed: () => onButtonPressed(TrackData.wheel),
          ),
          SideIconButton(
            selected: currentData,
            size: buttonSize,
            trackData: TrackData.map,
            onPressed: () => onButtonPressed(TrackData.map),
          ),
          SideIconButton(
            selected: currentData,
            size: buttonSize,
            trackData: TrackData.profile,
            onPressed: () => onButtonPressed(TrackData.profile),
          ),
        ],
      ),
    );
    Widget g = GestureDetector(
      onTap: onTap,
      child: Padding(
        padding: EdgeInsetsGeometry.fromLTRB(margin, margin, margin, margin),
        child: Stack(children: widgets),
      ),
    );
    //return g;
    return Padding(
      padding: EdgeInsetsGeometry.fromLTRB(0, 0, 5, 0),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxHeight: 200),
        child: Row(children: [Expanded(child: g), buttons]),
      ),
    );
  }
}

class Model extends ChangeNotifier {
  final Kinds kinds;
  final SegmentModel segment;
  Map<TrackData, FutureRenderer> map = {};
  TrackData current = TrackData.wheel;
  Model({required this.segment, required this.kinds}) {
    map[TrackData.map] = segment.makeRenderer(kinds, TrackData.map);
    map[TrackData.profile] = segment.makeRenderer(kinds, TrackData.profile);
    map[TrackData.wheel] = segment.makeRenderer(kinds, TrackData.wheel);
    segment.addListener(_onSegmentChanged);
  }

  void cycle() {
    if (current == TrackData.wheel) {
      return changeCurrent(TrackData.map);
    }
    if (current == TrackData.map) {
      return changeCurrent(TrackData.profile);
    }
    if (current == TrackData.profile) {
      return changeCurrent(TrackData.wheel);
    }
  }

  TrackData currentData() {
    return current;
  }

  // propagate changes in segmentModel.
  void _onSegmentChanged() {
    for (FutureRenderer r in map.values) {
      r.reset();
    }
    map[current]!.start();
    notifyListeners();
  }

  @override
  void dispose() {
    segment.removeListener(_onSegmentChanged);
    super.dispose();
  }

  void setSize(TrackData d, Size size) {
    developer.log("setSize: $d, current=$current");
    Size rendererSize = size;
    if (d != TrackData.wheel) {
      rendererSize = size * 1.25;
    }
    map[d]!.setSize(rendererSize);
    if (d == current) {
      _startCurrent();
    }
  }

  FutureRenderer renderer(TrackData d) {
    assert(map.containsKey(d));
    return map[d]!;
  }

  void _startCurrent() {
    FutureRenderer? r = map[current];
    assert(r != null);
    developer.log("startCurrent: ${r!.trackData}");
    if (r.needsStart()) {
      developer.log("start: ${r.trackData}");
      r.start();
    }
    // dont notifyListeners() because with are in build().
  }

  void changeCurrent(TrackData d) {
    current = d;
    _startCurrent();
    notifyListeners();
  }
}

class ProviderWidget extends StatelessWidget {
  final Kinds kinds;
  const ProviderWidget({super.key, required this.kinds});

  @override
  Widget build(BuildContext context) {
    SegmentModel segment = Provider.of<SegmentModel>(context);
    return ChangeNotifierProvider(
      create: (ctx) => Model(segment: segment, kinds: kinds),
      builder: (context, child) {
        return View(kinds: kinds);
      },
    );
  }
}

class TrackMultiView extends StatelessWidget {
  final Set<InputType> kinds;
  const TrackMultiView({super.key, required this.kinds});

  @override
  Widget build(BuildContext context) {
    return ProviderWidget(kinds: kinds);
  }
}
