import merge from 'lodash/merge';
import { defaultTheme } from 'react-admin';

export default merge({}, defaultTheme, {
  components: {
    MuiStack: {
      styleOverrides: {
        root: {
          display: 'flex',
          flexDirection: 'row',
          flexWrap: 'wrap',
          margin: '0',
          "& .ra-field": {
            padding: "0.5em",
            margin: "0.5em",
            background: "#eeeeee",
            borderRadius: "0.3em",
            flex: 'auto',
          },
          "& .ra-input": {
            padding: "0.2em",
            margin: "0.5em",
            flex: 'auto',
          },
          "& .ra-field-transaction": {
            maxWidth: '1500px',
            overflowWrap: 'break-word',
          },
          "& .flex-direction-row": {
            display: 'flex',
            flexDirection: 'row',
            flexWrap: 'wrap',
          },
          "& .flex-direction-column": {
            display: 'flex',
            flexDirection: 'column',
            flexWrap: 'wrap',
          },
          "& .button-kyc-request": {
            margin: '25px 10px 25px 20px',
          },
        }}
    },
    MuiTableBody: {
      styleOverrides: {
        root: {
          "& .column-id": {
            maxWidth: '400px',
            overflowWrap: 'break-word',
            wordBreak: 'break-all',
          },
          "& .column-createdAt": {
            minWidth: '100px',
          },
          "& .column-fundedAt": {
            minWidth: '100px',
          },
          "& .column-startedAt": {
            minWidth: '100px',
          },
          "& .column-hash": {
            width: '300px',
            overflowWrap: 'break-word',
            wordBreak: 'break-all',
          },
          "& .column-transactionHash": {
            width: '400px',
            overflowWrap: 'break-word',
            wordBreak: 'break-all',
          },
          "& .button-copy": {
            minWidth: '150px',
          },
        }}
    },
    MuiToolbar: {
      styleOverrides: {
        root: {
          "& button": {
            margin: '0 10px',
          },
          "& .custom-save-button": {
            padding: '10px',
          },
        }}
    },
    RaFormInput: {
      input: {
        width: "100%",
      }
    },
    RaCardContentInner: {
      styleOverrides: {
        root: {
          padding: '0',
        }
      }
    },
    RaLayout: {
      styleOverrides: {
        root: {
          padding: '10px',
          "& .warning-message": {
            color: '#aa1111',
            display: 'block',
            marginLeft: '20px',
          },
          "& .RaNullableBooleanInput-input": {
            minWidth: '200px',
          }
        }
      }
    },
    RaTabbedShowLayout: {
      styleOverrides: {
        root: {
          "& .ra-field": {
            display: 'flex',
            flexDirection: 'column',
            flexWrap: 'wrap',
            margin: '0',
          },
          "& .ra-field.nested-resource": {
            background: "none",
            padding: 0,
            margin: 0,
          }
        }
      }
    },
  },
});
