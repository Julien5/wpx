import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/trackviewswitch.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/trackview.dart';

class SegmentGraphicsButtons extends StatelessWidget {
  final VoidCallback? onPressed;
  final TrackData trackData;
  final TrackData selected;
  final double size;
  const SegmentGraphicsButtons({
    super.key,
    required this.selected,
    required this.size,
    required this.trackData,
    this.onPressed,
  });
  final double margin = 8;
  Image icon(TrackData data) {
    String filename = 'assets/icons/png/map.png';
    if (data == TrackData.wheel) {
      filename = 'assets/icons/png/clock.png';
    }
    if (data == TrackData.profile) {
      filename = 'assets/icons/png/profile.png';
    }
    if (data == TrackData.map) {
      filename = 'assets/icons/png/map.png';
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

class SegmentGraphicsButtonsColumn extends StatelessWidget {
  final void Function(TrackData) onButtonPressed;
  final TrackData selected;
  final double size;

  const SegmentGraphicsButtonsColumn({
    super.key,
    required this.selected,
    required this.size,
    required this.onButtonPressed,
  });

  @override
  Widget build(BuildContext context) {
    const double buttonSize = 30;
    return ConstrainedBox(
      constraints: const BoxConstraints(maxWidth: buttonSize),
      child: Column(
        mainAxisSize: MainAxisSize.max, // Makes Column fill available space
        mainAxisAlignment: MainAxisAlignment.spaceEvenly,
        children: [
          SegmentGraphicsButtons(
            selected: selected,
            size: buttonSize,
            trackData: TrackData.wheel,
            onPressed: () => onButtonPressed(TrackData.wheel),
          ),
          SegmentGraphicsButtons(
            selected: selected,
            size: buttonSize,
            trackData: TrackData.map,
            onPressed: () => onButtonPressed(TrackData.map),
          ),
          SegmentGraphicsButtons(
            selected: selected,
            size: buttonSize,
            trackData: TrackData.profile,
            onPressed: () => onButtonPressed(TrackData.profile),
          ),
        ],
      ),
    );
  }
}

class SegmentGraphics extends StatefulWidget {
  final Set<InputType> kinds;
  const SegmentGraphics({super.key, required this.kinds});

  @override
  State<SegmentGraphics> createState() => _SegmentGraphicsState();
}

class _SegmentGraphicsState extends State<SegmentGraphics>
    with AutomaticKeepAliveClientMixin {
  @override
  bool get wantKeepAlive => true; // This is crucial!

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
    TrackViewsSwitch model = Provider.of<TrackViewsSwitch>(
      context,
      listen: false,
    );
    model.cycle();
  }

  @override
  Widget build(BuildContext ctx) {
    super.build(ctx);
    // Instanciating a Provider.of<Model>(context) (listen=true)
    // is necessary to get rebuild on notifyListeners.
    TrackViewsSwitch model = Provider.of<TrackViewsSwitch>(context);
    double margin = 8;
    TrackData currentModelData = model.currentData();

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
    return GestureDetector(
      onTap: onTap,
      child: Padding(
        padding: EdgeInsetsGeometry.fromLTRB(margin, margin, margin, margin),
        child: visible,
      ),
    );
  }
}

class TrackGraphicsRow extends StatelessWidget {
  final Set<InputType> kinds;
  final double height;
  const TrackGraphicsRow({
    super.key,
    required this.kinds,
    required this.height,
  });

  void onButtonPressed(BuildContext context, TrackData data) {
    TrackViewsSwitch model = Provider.of<TrackViewsSwitch>(
      context,
      listen: false,
    );
    model.changeCurrent(data);
  }

  @override
  Widget build(BuildContext context) {
    // Instanciating a Provider.of<Model>(context) (listen=true)
    // is necessary to get rebuild on notifyListeners.
    TrackViewsSwitch model = Provider.of<TrackViewsSwitch>(context);
    TrackData currentModelData = model.currentData();

    Widget buttonColumn = SegmentGraphicsButtonsColumn(
      onButtonPressed: (trackData) => {onButtonPressed(context, trackData)},
      selected: currentModelData,
      size: 30,
    );

    return Padding(
      padding: EdgeInsetsGeometry.fromLTRB(0, 0, 5, 0),
      child: ConstrainedBox(
        constraints: BoxConstraints(maxHeight: height),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Expanded(child: SegmentGraphics(kinds: kinds)),
            buttonColumn,
          ],
        ),
      ),
    );
  }
}
