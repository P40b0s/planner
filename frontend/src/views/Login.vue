<template lang="pug">
.background
    .shape
    .shape
n-form
    h3 Авторизация
    n-form-item(label="Логин")
        n-input(type="text" size='large' v-model:value="login" placeholder="")
    n-form-item(label="Пароль")
        n-input(type="password" v-model:value="password" size='large' placeholder="")
    .login-error(v-if="!login_ok") Ошибка входа пароль или логин неверны
    n-button(@click="enter()") Вход
</template>
    
<script lang="ts">
import { ref, type Component, watch, inject, onMounted, onUnmounted, computed, onBeforeUnmount, h, toRefs } from 'vue';
import { type Events, type Emitter } from '../services/emitter';
import {NForm, NFormItem, NInput, NButton} from 'naive-ui'
//import  user_service  from '../services/user_service';
</script>
<script lang="ts" setup>
const emitter = inject<Emitter<Events>>('emitter') as Emitter<Events>;
    defineOptions({
    inheritAttrs: false
})
const login_ok = ref(true)
const login = ref("");
const password = ref("");
const enter = async () =>
{
    //login_ok.value = await user_service.login(login.value, password.value);
}
// const props = defineProps<{
//     document_id: string
// }>();

</script>
    
<style lang="scss" scoped>
.login-error
{
    color: #ff512f;
    font-size: 16px;
    height: 60px;
    border: 2px solid #d87676;

}
.background
{
    width: 400px;
    height: 420px;
    position: absolute;
    transform: translate(-50%,-50%);
    left: 50%;
    top: 50%;
}
.background .shape
{
    height: 200px;
    width: 200px;
    position: absolute;
    border-radius: 50%;
}
.shape:first-child
{
    background: linear-gradient(
        #1845ad,
        #23a2f6
    );
    left: -80px;
    top: -80px;
}
.shape:last-child
{
    background: linear-gradient(
        to right,
        #ff512f,
        #f09819
    );
    right: -30px;
    bottom: -80px;
}
form
{
    height: 400px;
    width: 400px;
    background-color: rgba(255,255,255,0.13);
    position: absolute;
    transform: translate(-50%,-50%);
    top: 50%;
    left: 50%;
    border-radius: 10px;
    backdrop-filter: blur(10px);
    border: 2px solid rgba(255,255,255,0.1);
    box-shadow: 0 0 40px rgba(8,7,16,0.6);
    padding: 50px 35px;
}
form *{
    font-family: 'Poppins',sans-serif;
    color: #ffffff;
    letter-spacing: 0.5px;
    outline: none;
    border: none;
}
form h3
{
    font-size: 32px;
    font-weight: 500;
    line-height: 42px;
    text-align: center;
}

n-form-item
{
    display: block;
    margin-top: 30px;
    font-size: 16px;
    font-weight: 500;
}



button
{
    margin-top: 10px;
    width: 100%;
    color: #080710;
    padding: 15px 0;
    font-size: 18px;
    font-weight: 600;
    border-radius: 5px;
    cursor: pointer;
}
.social
{
    margin-top: 30px;
    display: flex;
}
.social div
{
    background: red;
    width: 150px;
    border-radius: 3px;
    padding: 5px 10px 10px 5px;
    background-color: rgba(255,255,255,0.27);
    color: #eaf0fb;
    text-align: center;
}
.social div:hover
{
    background-color: rgba(255,255,255,0.47);
}
.social .fb
{
    margin-left: 25px;
}
.social i
{
    margin-right: 4px;
}
</style>