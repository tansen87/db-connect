<script setup lang="ts">
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { ElMessage } from "element-plus";

const loading = ref(false);
const form = reactive({
  host: "localhost",
  port: "3306",
  user: "root",
  pwd: "123456",
  db: "animalsystem",
  table: "relu",
  sqlsrc: "select * from relu",
  repcol: "E",
  outpath: "E:/Desktop/config/test_export",
});

listen("connErr", (event: any) => {
  const error: any = event.payload;
  const errmsg: any = "connect error: " + error;
  ElMessage.error(errmsg);
});

listen("errcode", (event: any) => {
  const errCode: any = event.payload;
  ElMessage.error(errCode);
});

listen("message", (event: any) => {
  const progress: any = event.payload;
  ElMessage.success(progress);
});

listen("queryErr", (event: any) => {
  const error: any = event.payload;
  const errmsg: any = "query error: " + error;
  ElMessage.error(errmsg);
});

// download data
async function getData() {
  if (form.host == "") {
    ElMessage.warning("host is empty.");
    return;
  }
  if (form.port == "") {
    ElMessage.warning("port is empty.");
    return;
  }
  if (form.user == "") {
    ElMessage.warning("user is empty.");
    return;
  }
  if (form.pwd == "") {
    ElMessage.warning("password is empty.");
    return;
  }
  if (form.db == "") {
    ElMessage.warning("database is empty.");
    return;
  }
  if (form.table == "") {
    ElMessage.warning("table is empty.");
    return;
  }
  if (form.sqlsrc == "") {
    ElMessage.warning("sql script is empty.");
    return;
  }
  if (form.outpath == "") {
    ElMessage.warning("output path is empty.");
    return;
  }
  if (
    form.host != "" &&
    form.port != "" &&
    form.user != "" &&
    form.pwd != "" &&
    form.db != "" &&
    form.table != "" &&
    form.sqlsrc != "" &&
    form.outpath != ""
  ) {
    ElMessage.info("downloading...");
    loading.value = true;
    await invoke("download", {
      host: form.host,
      port: form.port,
      user: form.user,
      pwd: form.pwd,
      db: form.db,
      table: form.table,
      sqlsrc: form.sqlsrc,
      repcol: form.repcol,
      outpath: form.outpath,
    });
    loading.value = false;
    ElMessage.success("download done.");
  }
}
</script>

<template>
  <el-form v-loading="loading" element-loading-text="Downloading...">
    <el-form :inline="true" :model="form" class="form-inline">
      <el-form-item label="host">
        <el-input v-model="form.host" placeholder="host" clearable />
      </el-form-item>
      <el-form-item label="port">
        <el-input v-model="form.port" placeholder="port" clearable />
      </el-form-item>
      <el-form-item label="user">
        <el-input v-model="form.user" placeholder="username" clearable />
      </el-form-item>
      <el-form-item label="pwd">
        <el-input v-model="form.pwd" placeholder="password" clearable />
      </el-form-item>
      <el-form-item label="db">
        <el-input v-model="form.db" placeholder="database" clearable />
      </el-form-item>
      <el-form-item label="table">
        <el-input v-model="form.table" placeholder="table" clearable />
      </el-form-item>
    </el-form>
    <el-form :model="form" :inline="true" class="form-text">
      <el-form-item>
        <el-input
          v-model="form.sqlsrc"
          :autosize="{ minRows: 2, maxRows: 5 }"
          type="textarea"
          placeholder="sql script"
          class="custom-textarea"
        />
      </el-form-item>
    </el-form>
    <el-form :inline="true" :model="form" class="form-out">
      <el-form-item label="output path">
        <el-input v-model="form.outpath" placeholder="output path" clearable />
      </el-form-item>
    </el-form>
    <el-form :inline="true" :model="form" class="form-out">
      <el-form-item label="replace col">
        <el-input
          v-model="form.repcol"
          placeholder="replace columns"
          clearable
        />
      </el-form-item>
    </el-form>
    <el-form :inline="true" :model="form" class="form-inline">
      <el-form-item>
        <el-button type="primary" @click="getData()">Download</el-button>
      </el-form-item>
    </el-form>
  </el-form>
</template>

<style>
.form-inline .el-input {
  --el-input-width: 220px;
}

.form-text .custom-textarea {
  width: 550px;
}

.form-out .el-input {
  --el-input-width: 470px;
}
</style>
