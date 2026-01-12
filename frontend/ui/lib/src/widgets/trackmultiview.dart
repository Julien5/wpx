import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
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

// Requires a SegmentModel.
// Makes a FutureRenderer with the given TrackData and Kinds.
// Not perfect, but hopefully okay.
class TrackView extends StatefulWidget {
  final RendererParameters parameters;
  const TrackView({super.key, required this.parameters});

  static TrackView make(Set<InputType> kinds, TrackData trackData) {
    RendererParameters p = RendererParameters(
      kinds: kinds,
      trackData: trackData,
    );
    return TrackView(key: p.createKey(), parameters: p);
  }

  @override
  State<TrackView> createState() => _TrackViewState();
}

class _TrackViewState extends State<TrackView> {
  FutureRenderer? renderer;
  SegmentModel? segmentModel;
  TrackData current = TrackData.wheel;

  void _onSegmentChanged() {
    if (renderer == null) {
      return;
    }
    setState(() {
      renderer!.reset();
    });
  }

  @override
  void initState() {
    super.initState();
    SchedulerBinding.instance.addPostFrameCallback((_) {
      _initState();
    });
  }

  @override
  void dispose() {
    if (segmentModel != null) {
      segmentModel!.removeListener(_onSegmentChanged);
    }
    super.dispose();
  }

  void _initState() {
    if (segmentModel == null) {
      segmentModel = Provider.of<SegmentModel>(context, listen: false);
      segmentModel!.addListener(_onSegmentChanged);
    }

    if (renderer == null) {
      assert(segmentModel != null);
      setState(() {
        renderer = segmentModel!.makeRenderer(
          widget.parameters.kinds,
          widget.parameters.trackData,
        );
      });
    }
  }

  @override
  Widget build(BuildContext ctx) {
    if (renderer == null) {
      return Center(
        child: Text("waiting for ${widget.parameters.trackData} renderer.."),
      );
    }
    return LayoutBuilder(
      builder: (BuildContext context, BoxConstraints constraints) {
        return FutureRenderingWidget(future: renderer!, interactive: false);
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
  Map<TrackData, TrackView> widgets = {};

  @override
  void initState() {
    super.initState();
    assert(widgets.isEmpty);
    for (TrackData data in {
      TrackData.profile,
      TrackData.map,
      TrackData.wheel,
    }) {
      widgets[data] = TrackView.make(widget.kinds, data);
    }
  }

  void onTap() {
    Model model = Provider.of<Model>(context, listen: false);
    model.cycle();
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
    Provider.of<Model>(context);
    double margin = 8;
    developer.log("rebuild view");
    TrackData currentModelData = currentTrackData();
    const double buttonSize = 30;
    Widget buttons = ConstrainedBox(
      constraints: const BoxConstraints(maxWidth: buttonSize),
      child: Column(
        mainAxisSize: MainAxisSize.max, // Makes Column fill available space
        mainAxisAlignment: MainAxisAlignment.spaceEvenly,
        children: [
          SideIconButton(
            selected: currentModelData,
            size: buttonSize,
            trackData: TrackData.wheel,
            onPressed: () => onButtonPressed(TrackData.wheel),
          ),
          SideIconButton(
            selected: currentModelData,
            size: buttonSize,
            trackData: TrackData.map,
            onPressed: () => onButtonPressed(TrackData.map),
          ),
          SideIconButton(
            selected: currentModelData,
            size: buttonSize,
            trackData: TrackData.profile,
            onPressed: () => onButtonPressed(TrackData.profile),
          ),
        ],
      ),
    );
    // I would like to have `visible = widgets[currentModelData]`
    // but then the widget states are disposed.
    // AI says: In Flutter, when you swap a widget out of the build tree,
    // the previous widget is unmounted and its State object is disposed of.
    // Solution: Stack with Offstaged widgets.
    Widget visible = Stack(
      fit: StackFit.expand, // <--- Add this line
      children:
          widgets.entries.map((entry) {
            return Offstage(
              offstage: entry.key != currentModelData,
              child: entry.value,
            );
          }).toList(),
    );
    Widget gesture = GestureDetector(
      onTap: onTap,
      child: Padding(
        padding: EdgeInsetsGeometry.fromLTRB(margin, margin, margin, margin),
        child: visible,
      ),
    );
    return Padding(
      padding: EdgeInsetsGeometry.fromLTRB(0, 0, 5, 0),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxHeight: 200),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [Expanded(child: gesture), buttons],
        ),
      ),
    );
  }
}

class Model extends ChangeNotifier {
  final Kinds kinds;
  TrackData current = TrackData.wheel;
  Model({required this.kinds});

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

  void changeCurrent(TrackData d) {
    current = d;
    notifyListeners();
  }
}

class ProviderWidget extends StatelessWidget {
  final Kinds kinds;
  const ProviderWidget({super.key, required this.kinds});

  @override
  Widget build(BuildContext context) {
    return ChangeNotifierProvider(
      create: (ctx) => Model(kinds: kinds),
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
