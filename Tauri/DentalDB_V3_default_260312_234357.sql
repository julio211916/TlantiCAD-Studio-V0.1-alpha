PRAGMA foreign_keys = OFF;

BEGIN;

SAVEPOINT dump;

CREATE TABLE client_allowedImportedIds (
    lab_id INT NOT NULL
  , practice_id INT NOT NULL
  , importedId TEXT
  , CONSTRAINT FKAD13DA5CBC06A0BF 
    FOREIGN KEY(lab_id, practice_id) REFERENCES clients
);

CREATE TABLE clients (
    lab_id INT NOT NULL
  , practice_id INT NOT NULL
  , name TEXT
  , language TEXT
  , country TEXT NOT NULL
  , city TEXT NOT NULL
  , zip TEXT NOT NULL
  , street TEXT NOT NULL
  , email TEXT
  , flags BIGINT
  , PRIMARY KEY(lab_id, practice_id)
  , UNIQUE(practice_id, lab_id)
  , CONSTRAINT FKE6F2B7C7292C12 
    FOREIGN KEY(lab_id) REFERENCES laboratories
);

INSERT INTO clients(rowid, lab_id, practice_id, name, language, country, city, zip, street, email, flags) VALUES(1, 0, 2147483647, 'MultiDiePractice', 'de', '', '', '', '', '', 0);
INSERT INTO clients(rowid, lab_id, practice_id, name, language, country, city, zip, street, email, flags) VALUES(2, 0, 1, 'Default', '', '', '', '', '', NULL, 0);

CREATE TABLE CustomWorkDefinition (
    id integer PRIMARY KEY AUTOINCREMENT
  , WorkTypeSN TEXT
  , MaterialSN TEXT
  , lab_id INT
  , practice_id INT
  , CONSTRAINT FKD2E715CABC06A0BF 
    FOREIGN KEY(lab_id, practice_id) REFERENCES clients
);

CREATE TABLE databaseInformation (
    id BLOB NOT NULL
  , created INTEGER
  , PRIMARY KEY(id)
);

INSERT INTO databaseInformation(rowid, id, created) VALUES(1, x'7bc0c48d4d386f4d9dce4973dc4d3bfc', 0);

CREATE TABLE DependentOnNumericToothWorkParameters (
    id integer PRIMARY KEY AUTOINCREMENT
  , parent_param_id INTEGER
  , dep_param_string TEXT
  , dep_param_value_string TEXT
  , dep_param_kind INTEGER
  , CONSTRAINT FK_A1E39E81 
    FOREIGN KEY(parent_param_id) REFERENCES NumericToothWorkParameter
);

CREATE TABLE DependentToothWorkParameters (
    id integer PRIMARY KEY AUTOINCREMENT
  , parent_param_id INT
  , dep_param_string TEXT
  , dep_param_value_string TEXT
  , dep_param_kind INTEGER
  , CONSTRAINT FK973A17B29C34DBCA 
    FOREIGN KEY(parent_param_id) REFERENCES TextualToothWorkParameter
);

CREATE TABLE laboratories (
    id INT NOT NULL
  , name TEXT
  , language TEXT
  , country TEXT NOT NULL
  , city TEXT NOT NULL
  , zip TEXT NOT NULL
  , street TEXT NOT NULL
  , PRIMARY KEY(id)
);

INSERT INTO laboratories(rowid, id, name, language, country, city, zip, street) VALUES(1, 0, 'Default', 'de', 'de', '', '', '');

CREATE TABLE NumericToothWorkParameter (
    id INT NOT NULL
  , ParameterSN TEXT
  , value DOUBLE
  , PRIMARY KEY(id)
  , CONSTRAINT FKEA1C3501469F64D8 
    FOREIGN KEY(id) REFERENCES ToothWorkParameters
);

CREATE TABLE patients (
    lab_id INT NOT NULL
  , practice_id INT NOT NULL
  , patient_id INT NOT NULL
  , fname TEXT
  , lname TEXT
  , dateOfBirth DATETIME
  , PRIMARY KEY(lab_id, practice_id, patient_id)
  , CONSTRAINT FK66884479BC06A0BF 
    FOREIGN KEY(lab_id, practice_id) REFERENCES clients
);

CREATE TABLE technicians (
    lab_id INT NOT NULL
  , tech_id INT NOT NULL
  , fname TEXT
  , lname TEXT
  , PRIMARY KEY(lab_id, tech_id)
  , CONSTRAINT FK9EC85DFE292C12 
    FOREIGN KEY(lab_id) REFERENCES laboratories
);

CREATE TABLE TextualToothWorkParameter (
    id INT NOT NULL
  , ParameterSN TEXT
  , ValueSN TEXT
  , PRIMARY KEY(id)
  , CONSTRAINT FK254F161F469F64D8 
    FOREIGN KEY(id) REFERENCES ToothWorkParameters
);

CREATE TABLE ToothWork (
    id integer PRIMARY KEY AUTOINCREMENT
  , WorkParams TEXT
  , WorkParamsFile TEXT
  , ToothId TEXT
  , flags BIGINT
  , WorkTypeSN TEXT
  , MaterialSN TEXT
  , treatment_id BIGINT
  , CONSTRAINT FK2EDE2B9A83A3B730 
    FOREIGN KEY(treatment_id) REFERENCES Treatment
);

CREATE TABLE ToothWorkParameters (
    id integer PRIMARY KEY AUTOINCREMENT
  , custom_wd_id INT
  , toothWork_id INT
  , CONSTRAINT FK2FC36C34A214A38E 
    FOREIGN KEY(custom_wd_id) REFERENCES CustomWorkDefinition
  , CONSTRAINT FK2FC36C34FF55443E 
    FOREIGN KEY(toothWork_id) REFERENCES ToothWork
);

CREATE TABLE Treatment (
    treatment_id integer PRIMARY KEY AUTOINCREMENT
  , practice_lab_id INT
  , practice_pr_id INT
  , patient_lab_id INT
  , patient_practice_id INT
  , patient_patient_id INT
  , tech_lab_id INT
  , tech_tech_id INT
  , t_date DATETIME
  , t_schaleId BIGINT
  , t_Notes TEXT
  , moonlighting INT
  , flags BIGINT
  , lockedby TEXT
  , imported_from_path TEXT
  , projectGUID TEXT
  , workParamsSHA TEXT
  , workParamsSignature TEXT
  , importedOrderId TEXT
  , importDebugInformation TEXT
  , workParamInfoId INT
  , t_duedate TEXT
  , ds_status TEXT
  , loggedInUserHashWhoCreatedTheCase TEXT
  , loggedInUserHashWhoLastSavedTheCase TEXT
  , CONSTRAINT FK36967990AB245501 
    FOREIGN KEY(practice_lab_id, practice_pr_id) REFERENCES clients
  , CONSTRAINT FK369679901E1F620B 
    FOREIGN KEY(patient_lab_id, patient_practice_id, patient_patient_id) REFERENCES patients
  , CONSTRAINT FK369679903060106A 
    FOREIGN KEY(tech_lab_id, tech_tech_id) REFERENCES technicians
  , CONSTRAINT FK369679909F838771 
    FOREIGN KEY(workParamInfoId) REFERENCES WorkParamsInfo
);

CREATE TABLE TreatmentValuedCustomInfo (
    id integer PRIMARY KEY AUTOINCREMENT
  , fieldId TEXT
  , Value TEXT
  , treatment_id BIGINT
  , CONSTRAINT FK7166B6C383A3B730 
    FOREIGN KEY(treatment_id) REFERENCES Treatment
);

CREATE TABLE TreatmentValuedParameters (
    id integer PRIMARY KEY AUTOINCREMENT
  , PARAM_TYPE TEXT NOT NULL
  , ParamSN TEXT
  , ValueSN TEXT
  , treatment_id BIGINT
  , CONSTRAINT FK9D0A809F83A3B730 
    FOREIGN KEY(treatment_id) REFERENCES Treatment
);

CREATE TABLE ValuedMaterialParameters (
    id integer PRIMARY KEY AUTOINCREMENT
  , PARAM_TYPE TEXT NOT NULL
  , ValueSN TEXT
  , MaterialSN TEXT
  , MaterialPropertySN TEXT
  , toothwork_id INT
  , CONSTRAINT FK6FB28652FF55443E 
    FOREIGN KEY(toothwork_id) REFERENCES ToothWork
);

CREATE TABLE WorkParamsData (
    hash TEXT NOT NULL
  , content BLOB
  , PRIMARY KEY(hash)
);

CREATE TABLE WorkParamsInfo (
    id integer PRIMARY KEY AUTOINCREMENT
  , workParamsDataHash TEXT
  , lastModificationDate DATETIME
  , CONSTRAINT FK52D6D2BB64B91A06 
    FOREIGN KEY(workParamsDataHash) REFERENCES WorkParamsData
);

CREATE TABLE WorkParamsInfoDentalShare (
    id INT NOT NULL
  , username TEXT
  , displayname TEXT
  , PRIMARY KEY(id)
  , CONSTRAINT FK4D89DB4C1F7F5D4 
    FOREIGN KEY(id) REFERENCES WorkParamsInfo
);

CREATE TABLE WorkParamsInfoLocal (
    id INT NOT NULL
  , fileName TEXT
  , PRIMARY KEY(id)
  , CONSTRAINT FK7526DA51F7F5D4 
    FOREIGN KEY(id) REFERENCES WorkParamsInfo
);

CREATE TABLE WorkParamsInfoLocalImport (
    id INT NOT NULL
  , PRIMARY KEY(id)
  , CONSTRAINT FKEE5A1C521F7F5D4 
    FOREIGN KEY(id) REFERENCES WorkParamsInfo
);

DELETE FROM sqlite_sequence;






CREATE INDEX idx_DependentOnNumericToothWorkParameters_parent_param_id ON DependentOnNumericToothWorkParameters(parent_param_id);

CREATE INDEX idx_DependentToothWorkParameters_parent_param_id ON DependentToothWorkParameters(parent_param_id);






CREATE INDEX idx_toothwork_treatment_id ON ToothWork(treatment_id);

CREATE INDEX idx_ToothWorkParameters_toothWork_id ON ToothWorkParameters(toothWork_id);



CREATE INDEX idx_TreatmentValuedParameters_treatment_id ON TreatmentValuedParameters(treatment_id);

CREATE INDEX idx_ValuedMaterialParameters_toothwork_id ON ValuedMaterialParameters(toothwork_id);





RELEASE dump;

COMMIT;

