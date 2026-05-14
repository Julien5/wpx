import 'dart:async';

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

String _formatDateTime(DateTime dt) {
  final d =
      '${dt.day.toString().padLeft(2, '0')} '
      '${_monthName(dt.month)} ${dt.year}';
  final t =
      '${dt.hour.toString().padLeft(2, '0')}:'
      '${dt.minute.toString().padLeft(2, '0')}';
  return '$d  ·  $t';
}

String _monthName(int m) =>
    const [
      '',
      'Jan',
      'Feb',
      'Mar',
      'Apr',
      'May',
      'Jun',
      'Jul',
      'Aug',
      'Sep',
      'Oct',
      'Nov',
      'Dec',
    ][m];

// ─── Public API ────────────────────────────────────────────────────────────

/// Shows the picker as a modal dialog and returns the chosen [DateTime],
/// or null if the user cancelled.
Future<DateTime?> showDateTimeRangePickerDialog({
  required BuildContext context,
  required DateTime min,
  required String minLabel,
  required DateTime max,
  required String maxLabel,
  DateTime? initial,
}) {
  DateTime min0 = DateTime(min.year, min.month, min.day, min.hour, min.minute);
  min0 = min0.add(Duration(minutes: 5));
  DateTime max0 = DateTime(max.year, max.month, max.day, max.hour, max.minute);
  max0 = max0.add(Duration(minutes: -5));
  return showDialog<DateTime>(
    context: context,
    builder:
        (_) => DateTimeRangePickerDialog(
          min: min0,
          minLabel: minLabel,
          max: max0,
          maxLabel: maxLabel,
          initial: initial,
        ),
  );
}

// ─── Dialog widget ─────────────────────────────────────────────────────────

class DateTimeRangePickerDialog extends StatefulWidget {
  const DateTimeRangePickerDialog({
    super.key,
    required this.min,
    required this.minLabel,
    required this.max,
    required this.maxLabel,
    this.initial,
  });

  final DateTime min;
  final String minLabel;
  final DateTime max;
  final String maxLabel;
  final DateTime? initial;

  @override
  State<DateTimeRangePickerDialog> createState() =>
      _DateTimeRangePickerDialogState();
}

class _DateTimeRangePickerDialogState extends State<DateTimeRangePickerDialog> {
  // Total range expressed in minutes
  late final int _totalMinutes;

  // Current selection expressed as minutes-from-min
  late int _offsetMinutes;

  // Fine-adjustment text controllers
  late final TextEditingController _hourCtrl;
  late final TextEditingController _minuteCtrl;

  // Day index relative to widget.min (0, 1, 2, …)
  int get _dayIndex => _current.difference(_dayStart(widget.min)).inDays;

  // List of distinct calendar days in the range
  late final List<DateTime> _days;

  @override
  void initState() {
    super.initState();
    _totalMinutes = widget.max.difference(widget.min).inMinutes;
    final initial = widget.initial ?? widget.min;
    _offsetMinutes = initial
        .difference(widget.min)
        .inMinutes
        .clamp(0, _totalMinutes);

    // Build the list of days
    _days = [];
    var d = _dayStart(widget.min);
    while (!d.isAfter(_dayStart(widget.max))) {
      _days.add(d);
      d = d.add(const Duration(days: 1));
    }

    _hourCtrl = TextEditingController(
      text: _current.hour.toString().padLeft(2, '0'),
    );
    _minuteCtrl = TextEditingController(
      text: _current.minute.toString().padLeft(2, '0'),
    );
  }

  @override
  void dispose() {
    _hourCtrl.dispose();
    _minuteCtrl.dispose();
    super.dispose();
  }

  DateTime get _current => widget.min.add(Duration(minutes: _offsetMinutes));

  DateTime _dayStart(DateTime dt) => DateTime(dt.year, dt.month, dt.day);

  /// Valid hour range for a given day index.
  ({int min, int max}) _hourBounds(int dayIdx) {
    final minH = dayIdx == 0 ? widget.min.hour : 0;
    final maxH = dayIdx == _days.length - 1 ? widget.max.hour : 23;
    return (min: minH, max: maxH);
  }

  /// Valid minute range for a given day index and hour.
  ({int min, int max}) _minuteBounds(int dayIdx, int hour) {
    final hb = _hourBounds(dayIdx);
    int minM = 0;
    int maxM = 59;
    if (dayIdx == 0 && hour == hb.min) minM = widget.min.minute;
    if (dayIdx == _days.length - 1 && hour == hb.max) {
      maxM = widget.max.minute;
    }
    return (min: minM, max: maxM);
  }

  void _applyOffset(int newOffset) {
    final clamped = newOffset.clamp(0, _totalMinutes);
    if (clamped == _offsetMinutes) return;
    setState(() {
      _offsetMinutes = clamped;
      _syncTextFields();
    });
  }

  void _syncTextFields() {
    _hourCtrl.text = _current.hour.toString().padLeft(2, '0');
    _minuteCtrl.text = _current.minute.toString().padLeft(2, '0');
  }

  /// Apply an arbitrary (day, hour, minute) triple, clamping to valid range.
  void _applyDHM(int dayIdx, int hour, int minute) {
    dayIdx = dayIdx.clamp(0, _days.length - 1);
    final hb = _hourBounds(dayIdx);
    hour = hour.clamp(hb.min, hb.max);
    final mb = _minuteBounds(dayIdx, hour);
    minute = minute.clamp(mb.min, mb.max);
    final target = _days[dayIdx].add(Duration(hours: hour, minutes: minute));
    final offset = target.difference(widget.min).inMinutes;
    _applyOffset(offset);
  }

  void _nudgeDay(int delta) =>
      _applyDHM(_dayIndex + delta, _current.hour, _current.minute);

  void _nudgeHour(int delta) =>
      _applyDHM(_dayIndex, _current.hour + delta, _current.minute);

  void _nudgeMinute(int delta) {
    _applyDHM(_dayIndex, _current.hour, _current.minute + delta);
  }

  void _commitHour() {
    final v = int.tryParse(_hourCtrl.text) ?? _current.hour;
    _applyDHM(_dayIndex, v, _current.minute);
    _hourCtrl.text = _current.hour.toString().padLeft(2, '0');
  }

  void _commitMinute() {
    final v = int.tryParse(_minuteCtrl.text) ?? _current.minute;
    _applyDHM(_dayIndex, _current.hour, v);
    _minuteCtrl.text = _current.minute.toString().padLeft(2, '0');
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final cs = theme.colorScheme;

    return Dialog(
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(20)),
      child: SizedBox(
        width: 400,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            _buildHeader(cs),
            const Divider(height: 1),
            _buildSliderSection(cs),
            const Divider(height: 1),
            _buildFineSection(cs),
            const Divider(height: 1),
            _buildFooter(cs),
          ],
        ),
      ),
    );
  }

  // ── Header ──────────────────────────────────────────────────────────────

  Widget _buildHeader(ColorScheme cs) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(20, 18, 20, 14),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'SELECT DATE & TIME',
            style: TextStyle(
              fontSize: 11,
              letterSpacing: 1.1,
              color: cs.onSurface.withValues(alpha: 0.45),
            ),
          ),
          const SizedBox(height: 4),
          Text(
            _formatDateTime(_current),
            style: TextStyle(
              fontSize: 22,
              fontWeight: FontWeight.w500,
              color: cs.onSurface,
              fontFeatures: const [FontFeature.tabularFigures()],
            ),
          ),
        ],
      ),
    );
  }

  // ── Coarse slider ────────────────────────────────────────────────────────

  Widget _buildSliderSection(ColorScheme cs) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(20, 14, 20, 10),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            'COARSE SELECTION',
            style: TextStyle(
              fontSize: 11,
              letterSpacing: 1.1,
              color: cs.onSurface.withValues(alpha: 0.45),
            ),
          ),
          const SizedBox(height: 8),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(widget.minLabel, style: _boundsStyle(cs)),
              Text(widget.maxLabel, style: _boundsStyle(cs)),
            ],
          ),
          const SizedBox(height: 2),
          SliderTheme(
            data: SliderTheme.of(context).copyWith(
              trackHeight: 4,
              thumbShape: const RoundSliderThumbShape(enabledThumbRadius: 8),
              overlayShape: const RoundSliderOverlayShape(overlayRadius: 16),
              activeTrackColor: cs.primary,
              inactiveTrackColor: cs.primary.withValues(alpha: 0.18),
              thumbColor: cs.primary,
              overlayColor: cs.primary.withValues(alpha: 0.12),
            ),
            child: Slider(
              min: 0,
              max: _totalMinutes.toDouble(),
              divisions: _totalMinutes,
              value: _offsetMinutes.toDouble(),
              onChanged: (v) => _applyOffset(v.round()),
            ),
          ),
          // Day-boundary pips
          _DayPips(
            days: _days,
            min: widget.min,
            totalMinutes: _totalMinutes,
            accentColor: cs.primary,
          ),
        ],
      ),
    );
  }

  // ── Fine spinners ────────────────────────────────────────────────────────

  Widget _buildFineSection(ColorScheme cs) {
    final hb = _hourBounds(_dayIndex);
    final mb = _minuteBounds(_dayIndex, _current.hour);

    return Padding(
      padding: const EdgeInsets.fromLTRB(20, 14, 20, 14),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'FINE ADJUSTMENT  ·  scroll or click to edit',
            style: TextStyle(
              fontSize: 11,
              letterSpacing: 1.1,
              color: cs.onSurface.withValues(alpha: 0.45),
            ),
          ),
          const SizedBox(height: 10),
          Row(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              // Day spinner
              Expanded(
                child: _SpinnerField(
                  label: 'Day',
                  displayText: '${_current.day} ${_monthName(_current.month)}',
                  isText: true,
                  onScroll: _nudgeDay,
                  onKeyArrow: _nudgeDay,
                ),
              ),
              const SizedBox(width: 8),
              // Hour spinner
              SizedBox(
                width: 76,
                child: _SpinnerField(
                  label: 'Hour',
                  controller: _hourCtrl,
                  minVal: hb.min,
                  maxVal: hb.max,
                  onScroll: _nudgeHour,
                  onKeyArrow: _nudgeHour,
                  onCommit: _commitHour,
                ),
              ),
              Padding(
                padding: const EdgeInsets.only(bottom: 10),
                child: Text(
                  ' : ',
                  style: TextStyle(
                    fontSize: 26,
                    fontWeight: FontWeight.w300,
                    color: cs.onSurface.withValues(alpha: 0.25),
                  ),
                ),
              ),
              // Minute spinner
              SizedBox(
                width: 76,
                child: _SpinnerField(
                  label: 'Minute',
                  controller: _minuteCtrl,
                  minVal: mb.min,
                  maxVal: mb.max,
                  onScroll: _nudgeMinute,
                  onKeyArrow: _nudgeMinute,
                  onCommit: _commitMinute,
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          // Hint showing current day's valid range
          Text(
            'Valid on ${_current.day} ${_monthName(_current.month)}: '
            '${hb.min.toString().padLeft(2, '0')}:${_minuteBounds(_dayIndex, hb.min).min.toString().padLeft(2, '0')}'
            ' – '
            '${hb.max.toString().padLeft(2, '0')}:${_minuteBounds(_dayIndex, hb.max).max.toString().padLeft(2, '0')}',
            style: TextStyle(
              fontSize: 11,
              color: cs.onSurface.withValues(alpha: 0.4),
            ),
          ),
        ],
      ),
    );
  }

  // ── Footer ───────────────────────────────────────────────────────────────

  Widget _buildFooter(ColorScheme cs) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(20, 12, 20, 16),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.end,
        children: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Cancel'),
          ),
          const SizedBox(width: 8),
          FilledButton(
            onPressed: () => Navigator.of(context).pop(_current),
            child: const Text('Confirm'),
          ),
        ],
      ),
    );
  }

  TextStyle _boundsStyle(ColorScheme cs) =>
      TextStyle(fontSize: 11, color: cs.onSurface.withValues(alpha: 0.4));

  String _formatShort(DateTime dt) =>
      '${dt.day} ${_monthName(dt.month)}  ${dt.hour.toString().padLeft(2, '0')}:${dt.minute.toString().padLeft(2, '0')}';
}

// ─── Day-boundary pip row ──────────────────────────────────────────────────

class _DayPips extends StatelessWidget {
  const _DayPips({
    required this.days,
    required this.min,
    required this.totalMinutes,
    required this.accentColor,
  });

  final List<DateTime> days;
  final DateTime min;
  final int totalMinutes;
  final Color accentColor;

  @override
  Widget build(BuildContext context) {
    // Boundary pips: one per day-transition (skip first day start = 0 %)
    final pips = <({double frac, String label})>[];
    for (int i = 1; i < days.length; i++) {
      final offsetMins = days[i].difference(min).inMinutes;
      pips.add((
        frac: offsetMins / totalMinutes,
        label: '${days[i].day} ${_monthName(days[i].month)}',
      ));
    }

    return LayoutBuilder(
      builder: (_, constraints) {
        final w = constraints.maxWidth;
        return SizedBox(
          height: 20,
          child: Stack(
            children:
                pips.map((p) {
                  return Positioned(
                    left: (p.frac * w - 20).clamp(0, w - 40),
                    top: 0,
                    child: Column(
                      children: [
                        Container(
                          width: 1.5,
                          height: 8,
                          color: accentColor.withValues(alpha: 0.6),
                        ),
                        Text(
                          p.label,
                          style: TextStyle(fontSize: 10, color: accentColor),
                        ),
                      ],
                    ),
                  );
                }).toList(),
          ),
        );
      },
    );
  }
}

/// A single numeric (or text) field that responds to:
///   • scroll wheel  → onScroll(+1 / -1)
///   • ↑ / ↓ keys   → onKeyArrow(+1 / -1)
///   • blur / Enter  → onCommit()
class _SpinnerField extends StatefulWidget {
  const _SpinnerField({
    required this.label,
    this.controller,
    this.displayText,
    this.isText = false,
    this.minVal,
    this.maxVal,
    required this.onScroll,
    required this.onKeyArrow,
    this.onCommit,
  });

  final String label;
  final TextEditingController? controller;
  final String? displayText; // used when isText = true
  final bool isText; // day field: non-editable, text display
  final int? minVal;
  final int? maxVal;
  final void Function(int delta) onScroll;
  final void Function(int delta) onKeyArrow;
  final VoidCallback? onCommit;

  @override
  State<_SpinnerField> createState() => _SpinnerFieldState();
}

class _SpinnerFieldState extends State<_SpinnerField> {
  double _scrollAccum = 0;
  final FocusNode _focus = FocusNode();
  Timer? _commitTimer;

  @override
  void dispose() {
    _commitTimer?.cancel();
    _focus.dispose();
    super.dispose();
  }

  void _scheduleCommit() {
    _commitTimer?.cancel();
    _commitTimer = Timer(const Duration(milliseconds: 250), () {
      widget.onCommit?.call();
    });
  }

  void _handleScroll(PointerScrollEvent ev) {
    _scrollAccum += ev.scrollDelta.dy;
    if (_scrollAccum.abs() >= 30) {
      widget.onScroll(_scrollAccum > 0 ? -1 : 1);
      _scrollAccum = 0;
    }
  }

  KeyEventResult _handleKey(FocusNode _, KeyEvent ev) {
    if (ev is! KeyDownEvent && ev is! KeyRepeatEvent) {
      return KeyEventResult.ignored;
    }
    if (ev.logicalKey == LogicalKeyboardKey.arrowUp) {
      widget.onKeyArrow(1);
      return KeyEventResult.handled;
    }
    if (ev.logicalKey == LogicalKeyboardKey.arrowDown) {
      widget.onKeyArrow(-1);
      return KeyEventResult.handled;
    }
    _scheduleCommit();
    return KeyEventResult.ignored;
  }

  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;

    final decoration = InputDecoration(
      labelText: widget.label,
      labelStyle: TextStyle(
        fontSize: 11,
        color: cs.onSurface.withValues(alpha: 0.5),
      ),
      contentPadding: const EdgeInsets.symmetric(horizontal: 10, vertical: 10),
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: BorderSide(
          color: cs.outline.withValues(alpha: 0.5),
          width: 0.5,
        ),
      ),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: BorderSide(
          color: cs.outline.withValues(alpha: 0.5),
          width: 0.5,
        ),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: BorderSide(color: cs.primary, width: 1.5),
      ),
    );

    final textStyle = TextStyle(
      fontSize: widget.isText ? 15 : 19,
      fontWeight: FontWeight.w500,
      color: cs.onSurface,
      fontFeatures: const [FontFeature.tabularFigures()],
    );

    return Listener(
      onPointerSignal: (ev) {
        if (ev is PointerScrollEvent) _handleScroll(ev);
      },
      child: MouseRegion(
        cursor:
            widget.isText
                ? SystemMouseCursors.resizeUpDown
                : SystemMouseCursors.resizeUpDown,
        child:
            widget.isText
                // Day field — read-only, shows text
                ? Focus(
                  focusNode: _focus,
                  onKeyEvent: _handleKey,
                  child: GestureDetector(
                    onTap: _focus.requestFocus,
                    child: InputDecorator(
                      decoration: decoration,
                      child: Text(
                        widget.displayText ?? '',
                        style: textStyle,
                        textAlign: TextAlign.center,
                      ),
                    ),
                  ),
                )
                // Hour / minute field — editable number
                : Focus(
                  onKeyEvent: _handleKey,
                  child: TextFormField(
                    controller: widget.controller,
                    focusNode: _focus,
                    decoration: decoration,
                    style: textStyle,
                    textAlign: TextAlign.center,
                    keyboardType: TextInputType.number,
                    inputFormatters: [
                      // 1. Allow only digits
                      FilteringTextInputFormatter.digitsOnly,
                      // 2. Prevent more than 2 characters
                      LengthLimitingTextInputFormatter(2),
                      /*_RangeFormatter(
                        min: widget.minVal ?? 0,
                        max: widget.maxVal ?? 59,
                      ),*/
                    ],
                    onEditingComplete: () {
                      widget.onCommit?.call();
                      _focus.unfocus();
                    },
                    onTapOutside: (_) {
                      widget.onCommit?.call();
                      _focus.unfocus();
                    },
                  ),
                ),
      ),
    );
  }
}

// ─── Input formatter: clamp on keystroke ──────────────────────────────────

class _RangeFormatter extends TextInputFormatter {
  const _RangeFormatter({required this.min, required this.max});
  final int min;
  final int max;

  @override
  TextEditingValue formatEditUpdate(TextEditingValue _, TextEditingValue next) {
    if (next.text.isEmpty) return next;
    final v = int.tryParse(next.text);
    if (v == null) return next;
    final clamped = v.clamp(min, max);
    if (clamped != v) {
      final s = clamped.toString().padLeft(2, '0');
      return TextEditingValue(
        text: s,
        selection: TextSelection.collapsed(offset: s.length),
      );
    }
    return next;
  }
}
