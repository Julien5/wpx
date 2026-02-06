import 'dart:io';
import 'package:file_picker/file_picker.dart';
import 'package:file_saver/file_saver.dart';
import 'package:flutter/foundation.dart';
import 'package:ui/src/models/root.dart';

enum Type { pdf, gpx }

String fileExtension(Type type) {
  if (type == Type.pdf) {
    return "pdf";
  }
  return "gpx";
}

MimeType mimeType(Type type) {
  if (type == Type.pdf) {
    return MimeType.pdf;
  }
  return MimeType.custom;
}

FileType fileType(Type type) {
  if (type == Type.pdf) {
    return FileType.any;
  }
  return FileType.any;
}

void fileSave(List<int> data, Type type) async {
  if (kIsWeb) {
    await FileSaver.instance.saveFile(
      name: "waypoints", // on the web, the extension is set automatically...
      bytes: Uint8List.fromList(data),
      fileExtension: fileExtension(type),
      mimeType: mimeType(type),
      customMimeType: fileExtension(type),
    );
  } else if (Platform.isLinux) {
    var filepath = await FilePicker.platform.saveFile(
      fileName: "waypoints.${fileExtension(type)}", // .. but not on linux
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
  assert(type == Type.gpx);
  var data = await root.generateGpx();
  return data;
}
