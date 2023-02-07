import { Pagination } from 'react-admin'
import Countrily from "countrily";
import JSZip from "jszip";
import { saveAs } from "file-saver";
  
export const PostPagination = props => <Pagination rowsPerPageOptions={[20]} {...props} />;

export const defaultSort = { field: 'id', order: 'DESC' };

export const documentSort = { field: 'createdAt', order: 'DESC' };

export const convertBase64 = (file) => {
  return new Promise((resolve, reject) => {
    const fileReader = new FileReader();
    fileReader.readAsDataURL(file);

    fileReader.onload = () => {
      resolve(fileReader.result);
    };

    fileReader.onerror = (error) => {
      reject(error);
    };
  });
};

export const demonymList = () => {
  var demonyms = [];
  Countrily.all().map(value => {
    if (!value.demonym) return false;
    if (demonyms.filter(f => f.name === value.demonym).length > 0) return false;
    demonyms.push({ name: value.demonym})
    return false;
  })
  return demonyms;
}

export const parseDate = (date) => {
  return new Date(date).toLocaleString([], {
    timeStyle: "short",
    dateStyle: "medium"
  });
}

export const createZipAndDownload = async (files, kycRequestId) => {
  var zip = new JSZip();
  for (let file of files) {
    const base64Response = await fetch(`data:${file.contentType};base64,${file.payload}`);
    let blob = await base64Response.blob();
    zip.file(file.filename, blob);
  }
  let zipName = "evidence_" + kycRequestId + ".zip";
  return zip.generateAsync({type:"blob"})
  .then(function (blob) {
      saveAs(blob, zipName);
  });
}