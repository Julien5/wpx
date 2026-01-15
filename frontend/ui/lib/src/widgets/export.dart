import 'dart:developer' as developer;
import 'dart:io';
import 'package:file_picker/file_picker.dart';
import 'package:file_saver/file_saver.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';

enum Type { pdf, gpx, zip }

String fileExtension(Type type) {
  if (type == Type.pdf) {
    return "pdf";
  }
  if (type == Type.zip) {
    return "zip";
  }
  return "gpx";
}

MimeType mimeType(Type type) {
  if (type == Type.pdf) {
    return MimeType.pdf;
  }
  if (type == Type.zip) {
    return MimeType.zip;
  }
  return MimeType.custom;
}

FileType fileType(Type type) {
  if (type == Type.pdf) {
    return FileType.any;
  }
  if (type == Type.zip) {
    return FileType.any;
  }
  return FileType.any;
}

void fileSave(List<int> data, Type type) async {
  if (kIsWeb) {
    await FileSaver.instance.saveFile(
      name: "route", // on the web, the extension is set automatically...
      bytes: Uint8List.fromList(data),
      fileExtension: fileExtension(type),
      mimeType: mimeType(type),
      customMimeType: fileExtension(type),
    );
  } else if (Platform.isLinux) {
    var filepath = await FilePicker.platform.saveFile(
      fileName: "route.${fileExtension(type)}", // .. but not on linux
      type: fileType(type),
      allowedExtensions: [fileExtension(type)],
      bytes: Uint8List.fromList(data),
    );
    if (filepath == null) {
      return;
    }
    await Process.run('xdg-open', [filepath]);
  }
}

Future<List<int>> generate(RootModel root, Type type) async {
  if (type == Type.pdf) {
    var data = await root.generatePdf();
    return data;
  }
  if (type == Type.zip) {
    var data = await root.generateZip();
    return data;
  }
  assert(type == Type.gpx);
  var data = await root.generateGpx();
  return data;
}

class ExportButton extends StatefulWidget {
  final Type type;
  final String text;
  const ExportButton({super.key, required this.type, required this.text});

  @override
  State<ExportButton> createState() => _ExportButtonState();
}

class _ExportButtonState extends State<ExportButton> {
  int length = 0;

  void onPressed(RootModel root) async {
    if (!mounted) {
      return;
    }
    setState(() {
      length = 0;
    });
    var data = await generate(root, widget.type);
    fileSave(data, widget.type);
  }

  @override
  Widget build(BuildContext context) {
    RootModel model = Provider.of<RootModel>(context);
    return Row(
      children: [
        ElevatedButton(
          onPressed: () => onPressed(model),
          child: Text(widget.text),
        ),
      ],
    );
  }
}
