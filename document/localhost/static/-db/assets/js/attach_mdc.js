window.addEventListener('DOMContentLoaded', (event) => {
    document.querySelectorAll('.mdc-text-field').forEach(function (e) {
        mdc.textField.MDCTextField.attachTo(e);
    });
    document.querySelectorAll('.mdc-button').forEach(function (e) {
        mdc.ripple.MDCRipple.attachTo(e);
    });
    document.querySelectorAll('.mdc-fab').forEach(function (e) {
        mdc.ripple.MDCRipple.attachTo(e);
    });
    document.querySelectorAll('.mdc-deprecated-list').forEach(function (e) {
        const list = mdc.list.MDCList.attachTo(e);
        list.wrapFocus = true;
        mdc.ripple.MDCRipple.attachTo(e);
    });
    document.querySelectorAll('.mdc-drawer').forEach(function (e) {
        const drawer = mdc.drawer.MDCDrawer.attachTo(e);
        document.querySelectorAll('.mdc-top-app-bar').forEach(function (e) {
            let topAppBar = mdc.topAppBar.MDCTopAppBar.attachTo(e);
            topAppBar.setScrollTarget(document.querySelector('.mdc-drawer-app-content'));
            topAppBar.listen('MDCTopAppBar:nav', () => {
                drawer.open = !drawer.open;
            });
            document.addEventListener("click",(event)=>{
                if(drawer.open&&!event.target.closest('.mdc-drawer')){
                    drawer.open = false;
                }
            })
        });
    });
    document.querySelectorAll('.mdc-data-table').forEach(function (e) {
        mdc.dataTable.MDCDataTable.attachTo(e);
    });
    
});