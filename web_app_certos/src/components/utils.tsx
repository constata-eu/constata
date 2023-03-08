import { Pagination, TopToolbar, CreateButton } from 'react-admin'
import * as React from 'react';
import { getRawAuthorization } from './auth_provider';
import BootstrapTooltip from './tooltip_blue';

export const defaultSort = { field: 'id', order: 'DESC' };

export const PaginationDefault = props => <Pagination rowsPerPageOptions={[20]} {...props} />;

export const ListCreateActions = (props) => {
  const [open, setOpen] = React.useState(false);
  const handleOpen = (e) => {
    e.target.focus();
    setOpen(true)
  };
  const handleClose = (e) => {
    e.target.blur();
    setOpen(false)
  };
  
  return (
    <TopToolbar>
      <BootstrapTooltip
        {...props}
        placement="left"
        leaveDelay={200}
        open={open}
        onMouseEnter={handleOpen}
        onMouseLeave={handleClose}
        arrow>
          <CreateButtonRef sx={{ zIndex: 10000}} />
      </BootstrapTooltip>
    </TopToolbar> 
)};


const CreateButtonRef = React.forwardRef((props: any, _) => (
    <CreateButton {...props} />
));

export const ListActionsWithoutCreate = () => {
  return (
    <TopToolbar>
    </TopToolbar> 
)};


interface base64 {
  (file: any): Promise<string>;
}

export const convertBase64: base64 = (file) => {
  return new Promise((resolve, reject) => {
    const fileReader = new FileReader();
    fileReader.readAsDataURL(file);

    fileReader.onload = () => {
      if (typeof fileReader.result === "string") {
        resolve(fileReader.result);
      };
    };

    fileReader.onerror = (error) => {
      reject(error);
    };
  });
};

export const readValuesAsText = (file) => {
  return new Promise((resolve, reject) => {
    const fileReader = new FileReader();
    fileReader.readAsText(file);

    fileReader.onload = () => {
      resolve(fileReader.result);
    };

    fileReader.onerror = (error) => {
      reject(error);
    };
  });
};

export const downloadFile = async (path, filename, notify) =>
  {
  const proof_url = `${process.env.REACT_APP_CERTOS_API_DOMAIN || ''}${path}`;
  const headers: any =  { 'Authentication': await getRawAuthorization(proof_url, "GET", null) };
  const response = await fetch(proof_url, {headers});

  if(!response.ok) {
    return notify("certos.errors.downloadPayload");
  }
  let blob = await response.blob();
  openBlob(blob, filename);
}

export const openBlob = async (blob: any, filename?: string) => {
  const a = document.createElement('a');
  document.body.appendChild(a);
  const url = window.URL.createObjectURL(blob);
  a.href = url;
  if(filename) {
    a.download = filename;
  } else {
    a.target = "_blank";
  }
  a.click();
  setTimeout(() => {
    window.URL.revokeObjectURL(url);
    document.body.removeChild(a);
  }, 0)
}

export const parseDate = (date) => {
  return new Date(date).toLocaleString([], {
    timeStyle: "short",
    dateStyle: "medium"
  });
}

export function formatJsonInline(params){
  return <pre>
    { Object.entries(orderJsonObject(JSON.parse(params))).map(([key, value] : [string, string]) =>
    <span key={key} className="params">
        <span><b>{key}</b>: </span>
        <span>{value}</span>
        <br/>
    </span>
    ) }
  </pre>
}

const orderJsonObject = (unorderedObject) => {
  return Object.keys(unorderedObject).sort().reduce(
    (obj, key) => { 
      obj[key] = unorderedObject[key]; 
      return obj;
    }, 
    {}
  );
}

export const handleErrors = (error, notify) => {
  switch (error?.body?.message) {
    case "ValidationError on address: not_an_email":
      notify('certos.errors.not_an_email', {type: 'warning'});
      break;
    default:
      notify('certos.errors.default', {type: 'warning'});
      break;
  }
}

export class BaseError extends Error {
  statusCode: number;

  constructor(statusCode: number, message: string) {
    super(message);

    Object.setPrototypeOf(this, new.target.prototype);
    this.name = Error.name;
    this.statusCode = statusCode;
    Error.captureStackTrace(this);
  }
}

export const handleBoundingClientRect = (resource: string) => {
  let element = document.querySelector(`[role='menuitem'][href='#/${resource}']`)?.getBoundingClientRect();
  return new DOMRect(element?.x, element?.y, element?.width, element?.height ? element.height - 10 : 0);
}
