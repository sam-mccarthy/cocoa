db = db.getSiblingDB('cocoa');

db.createUser({
    user: 'cocoa',
    pwd: 'default',
    roles: [{role: 'readWrite', db: 'cocoa'}]
});